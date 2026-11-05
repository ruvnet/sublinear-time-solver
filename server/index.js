const express = require('express');
const cors = require('cors');
const compression = require('compression');
const helmet = require('helmet');
const rateLimit = require('express-rate-limit');
const { WebSocketServer } = require('ws');
const { EventEmitter } = require('events');
const crypto = require('crypto');
const { v4: uuidv4 } = require('uuid');
const { StreamingManager } = require('./streaming');
const { SessionManager } = require('./session-manager');
const { FlowNexusIntegration } = require('../integrations/flow-nexus');

class SolverServer extends EventEmitter {
  constructor(options = {}) {
    super();

    this.config = {
      port: options.port || 3000,
      cors: options.cors || false,
      // Explicit origin allowlist for credentialed CORS. When empty, we never
      // reflect an arbitrary Origin together with credentials (see setupMiddleware).
      corsOrigins: options.corsOrigins || null,
      workers: options.workers || 1,
      maxSessions: options.maxSessions || 100,
      // Hard upper bound on any matrix/vector dimension accepted from a request.
      // Prevents attacker-declared dimensions from driving unbounded allocations.
      maxDimension: options.maxDimension || 1000000,
      // Cap on total dense elements (rows*cols) and on sparse non-zeros. A
      // per-axis cap alone lets a 1e6 x 1e6 body through — O(rows*cols) work
      // per solver sweep pins the worker. These bound the actual work/memory.
      maxElements: options.maxElements || 25000000,     // ~200MB as f64
      maxNonZeros: options.maxNonZeros || 5000000,
      // Cap on solver iterations requested per job (compounds worker starvation).
      maxIterations: options.maxIterations || 100000,
      authToken: options.authToken,
      flowNexusEnabled: options.flowNexusEnabled || false,
      ...options
    };

    this.app = express();
    this.server = null;
    this.wss = null;

    this.sessions = new SessionManager(this.config);
    this.streaming = new StreamingManager(this.config);
    this.flowNexus = this.config.flowNexusEnabled ? new FlowNexusIntegration() : null;

    this.setupMiddleware();
    this.setupRoutes();
    this.setupWebSocket();
  }

  setupMiddleware() {
    // Security middleware
    this.app.use(helmet({
      contentSecurityPolicy: false, // Allow WebSocket connections
      crossOriginEmbedderPolicy: false
    }));

    // CORS
    if (this.config.cors) {
      const allowlist = Array.isArray(this.config.corsOrigins)
        ? this.config.corsOrigins
        : (this.config.corsOrigins ? [this.config.corsOrigins] : null);
      // Only pair credentials with an explicit origin allowlist. Reflecting an
      // arbitrary Origin (`origin: true`) while allowing credentials lets any
      // website make credentialed cross-origin reads once auth is added.
      this.app.use(cors({
        origin: allowlist || false,
        credentials: allowlist !== null,
        methods: ['GET', 'POST', 'PUT', 'DELETE', 'OPTIONS'],
        allowedHeaders: ['Content-Type', 'Authorization', 'X-Session-ID']
      }));
    }

    // Compression
    this.app.use(compression());

    // Rate limiting
    const limiter = rateLimit({
      windowMs: 15 * 60 * 1000, // 15 minutes
      max: 1000, // Limit each IP to 1000 requests per windowMs
      message: {
        error: 'Too many requests',
        retryAfter: '15 minutes'
      }
    });
    this.app.use('/api', limiter);

    // Body parsing
    this.app.use(express.json({ limit: '50mb' }));
    this.app.use(express.urlencoded({ extended: true, limit: '50mb' }));

    // Request logging
    this.app.use((req, res, next) => {
      const timestamp = new Date().toISOString();
      console.log(`[${timestamp}] ${req.method} ${req.path}`);
      next();
    });

    // Authentication middleware. Mounted on the real API prefix (`/api/v1`),
    // not an unused `/api/protected` path — otherwise `--auth-token` never
    // actually gated any endpoint. No-op when authToken is unset.
    this.app.use('/api/v1', this.authenticateToken.bind(this));
  }

  setupRoutes() {
    // Health check
    this.app.get('/health', (req, res) => {
      res.json({
        status: 'healthy',
        timestamp: new Date().toISOString(),
        uptime: process.uptime(),
        memory: process.memoryUsage(),
        sessions: this.sessions.getStats()
      });
    });

    // API routes
    this.app.use('/api/v1', this.createAPIRoutes());

    // Static files for documentation
    this.app.use('/docs', express.static('docs'));

    // Root endpoint
    this.app.get('/', (req, res) => {
      res.json({
        name: 'Sublinear Time Solver API',
        version: '1.0.0',
        endpoints: {
          health: '/health',
          api: '/api/v1',
          docs: '/docs',
          websocket: '/ws'
        }
      });
    });
  }

  createAPIRoutes() {
    const router = express.Router();

    // Start streaming solve session
    router.post('/solve-stream', async (req, res) => {
      try {
        const {
          matrix,
          vector,
          method = 'adaptive',
          options = {},
          flow_nexus = {}
        } = req.body;

        if (!matrix || !vector) {
          return res.status(400).json({
            error: 'Matrix and vector are required',
            code: 'MISSING_INPUT'
          });
        }

        const sizeError = this.validateProblemSize(matrix, vector);
        if (sizeError) {
          return res.status(400).json({ error: sizeError, code: 'INVALID_DIMENSIONS' });
        }

        const sessionId = uuidv4();
        const session = await this.sessions.createSession(sessionId, {
          matrix,
          vector,
          method,
          options: this.sanitizeOptions(options),
          flowNexus: flow_nexus
        });

        res.writeHead(200, {
          'Content-Type': 'application/x-ndjson',
          'Cache-Control': 'no-cache',
          'Connection': 'keep-alive',
          'X-Session-ID': sessionId
        });

        // Start streaming solve
        const stream = await this.streaming.startSolve(session);

        // Track client disconnect so we stop pulling from the solver and let
        // the async iterator's finally release the worker (see streaming.js).
        let clientGone = false;
        res.on('close', () => { clientGone = true; });

        for await (const update of stream) {
          if (clientGone) break;
          const data = JSON.stringify({
            type: 'iteration_update',
            session_id: sessionId,
            timestamp: new Date().toISOString(),
            ...update
          }) + '\n';

          if (!res.write(data)) {
            // Backpressure: resume on drain OR on client disconnect, so a
            // closed socket (drain never fires) can't hang the handler and
            // pin the worker.
            await new Promise((resolve) => {
              const done = () => { res.removeListener('drain', done); res.removeListener('close', done); resolve(); };
              res.once('drain', done);
              res.once('close', done);
            });
          }

          if (clientGone || update.converged || update.error) {
            break;
          }
        }

        res.end();

      } catch (error) {
        console.error('Solve stream error:', error);
        res.status(500).json({
          error: error.message,
          code: 'SOLVE_ERROR'
        });
      }
    });

    // Submit solve job
    router.post('/solve', async (req, res) => {
      try {
        const { matrix, vector, method, options } = req.body;

        if (!matrix || !vector) {
          return res.status(400).json({
            error: 'Matrix and vector are required'
          });
        }

        const sizeError = this.validateProblemSize(matrix, vector);
        if (sizeError) {
          return res.status(400).json({ error: sizeError, code: 'INVALID_DIMENSIONS' });
        }

        const jobId = await this.sessions.submitJob({
          matrix,
          vector,
          method,
          options: this.sanitizeOptions(options)
        });

        res.json({
          job_id: jobId,
          status: 'submitted',
          endpoints: {
            status: `/api/v1/jobs/${jobId}`,
            stream: `/api/v1/jobs/${jobId}/stream`
          }
        });

      } catch (error) {
        console.error('Submit job error:', error);
        res.status(500).json({
          error: error.message,
          code: 'SUBMIT_ERROR'
        });
      }
    });

    // Get job status
    router.get('/jobs/:jobId', async (req, res) => {
      try {
        const status = await this.sessions.getJobStatus(req.params.jobId);
        if (!status) {
          return res.status(404).json({
            error: 'Job not found',
            code: 'JOB_NOT_FOUND'
          });
        }

        res.json(status);

      } catch (error) {
        console.error('Get job status error:', error);
        res.status(500).json({
          error: error.message,
          code: 'STATUS_ERROR'
        });
      }
    });

    // Stream job updates
    router.get('/jobs/:jobId/stream', async (req, res) => {
      try {
        const stream = await this.sessions.getJobStream(req.params.jobId);
        if (!stream) {
          return res.status(404).json({
            error: 'Job not found or not streaming'
          });
        }

        res.writeHead(200, {
          'Content-Type': 'text/event-stream',
          'Cache-Control': 'no-cache',
          'Connection': 'keep-alive'
        });

        for await (const update of stream) {
          res.write(`data: ${JSON.stringify(update)}\n\n`);
        }

        res.end();

      } catch (error) {
        console.error('Job stream error:', error);
        res.status(500).json({
          error: error.message,
          code: 'STREAM_ERROR'
        });
      }
    });

    // Verification endpoint
    router.post('/verify', async (req, res) => {
      try {
        const {
          session_id,
          probe_count = 10,
          tolerance = 1e-8
        } = req.body;

        if (!session_id) {
          return res.status(400).json({
            error: 'Session ID is required'
          });
        }

        const result = await this.sessions.verifySession(session_id, {
          probeCount: probe_count,
          tolerance
        });

        res.json({
          type: 'verification_result',
          session_id,
          timestamp: new Date().toISOString(),
          ...result
        });

      } catch (error) {
        console.error('Verification error:', error);
        res.status(500).json({
          error: error.message,
          code: 'VERIFICATION_ERROR'
        });
      }
    });

    // Session status
    router.get('/sessions/:sessionId', async (req, res) => {
      try {
        const session = await this.sessions.getSession(req.params.sessionId);
        if (!session) {
          return res.status(404).json({
            error: 'Session not found'
          });
        }

        res.json({
          session_id: req.params.sessionId,
          status: session.status,
          created_at: session.createdAt,
          last_activity: session.lastActivity,
          metrics: session.metrics
        });

      } catch (error) {
        console.error('Session status error:', error);
        res.status(500).json({
          error: error.message,
          code: 'SESSION_ERROR'
        });
      }
    });

    // Cost update endpoint for swarm coordination
    router.post('/swarm/costs', async (req, res) => {
      try {
        const {
          session_id,
          delta_costs,
          matrix_updates,
          source_node
        } = req.body;

        if (!session_id || !delta_costs) {
          return res.status(400).json({
            error: 'Session ID and delta costs are required'
          });
        }

        await this.sessions.updateCosts(session_id, {
          deltaCosts: delta_costs,
          matrixUpdates: matrix_updates,
          sourceNode: source_node
        });

        res.json({
          status: 'updated',
          timestamp: new Date().toISOString()
        });

      } catch (error) {
        console.error('Cost update error:', error);
        res.status(500).json({
          error: error.message,
          code: 'COST_UPDATE_ERROR'
        });
      }
    });

    // Swarm join endpoint
    router.post('/swarm/join', async (req, res) => {
      try {
        const {
          session_id,
          node_id,
          capabilities = []
        } = req.body;

        if (!session_id || !node_id) {
          return res.status(400).json({
            error: 'Session ID and node ID are required'
          });
        }

        await this.sessions.joinSwarm(session_id, {
          nodeId: node_id,
          capabilities
        });

        res.json({
          status: 'joined',
          node_id,
          session_id
        });

      } catch (error) {
        console.error('Swarm join error:', error);
        res.status(500).json({
          error: error.message,
          code: 'SWARM_JOIN_ERROR'
        });
      }
    });

    // Flow-Nexus integration endpoints
    if (this.flowNexus) {
      router.post('/flow-nexus/register', async (req, res) => {
        try {
          const result = await this.flowNexus.registerSolver({
            endpoint: `http://localhost:${this.config.port}`,
            capabilities: ['streaming', 'verification', 'swarm']
          });

          res.json(result);

        } catch (error) {
          console.error('Flow-Nexus registration error:', error);
          res.status(500).json({
            error: error.message,
            code: 'FLOW_NEXUS_ERROR'
          });
        }
      });

      router.get('/flow-nexus/status', async (req, res) => {
        try {
          const status = await this.flowNexus.getStatus();
          res.json(status);

        } catch (error) {
          console.error('Flow-Nexus status error:', error);
          res.status(500).json({
            error: error.message,
            code: 'FLOW_NEXUS_ERROR'
          });
        }
      });
    }

    return router;
  }

  setupWebSocket() {
    // WebSocket setup will be done when server starts
  }

  /**
   * Validate that a matrix/vector payload declares sane, bounded dimensions
   * before any session/job is created. Returns an error string, or null if ok.
   * Guards against tiny sparse bodies that declare enormous dimensions
   * (e.g. rows: 2e9) which would drive unbounded allocations downstream.
   */
  validateProblemSize(matrix, vector) {
    const max = this.config.maxDimension;

    const isPositiveInt = (v) => Number.isInteger(v) && v > 0;
    const arrayLen = (v) => (Array.isArray(v) || ArrayBuffer.isView(v) ? v.length : undefined);
    let rows;
    let cols;
    let isDense = false;
    let nnz; // number of stored entries for a sparse matrix, if determinable

    if (Array.isArray(matrix)) {
      // Dense: array of rows.
      isDense = true;
      rows = matrix.length;
      cols = Array.isArray(matrix[0]) ? matrix[0].length : rows;
    } else if (matrix && typeof matrix === 'object') {
      // Sparse (COO/CSR) or {data, rows, cols} shape.
      rows = matrix.rows ?? matrix.size ?? matrix.dimension;
      cols = matrix.cols ?? matrix.size ?? matrix.dimension ?? rows;
      const data = matrix.data;
      if (Array.isArray(data) && Array.isArray(data[0])) {
        // dense-as-object (data is an array of rows)
        isDense = true;
      } else {
        // Count stored entries from the common COO/CSR value carriers.
        nnz = arrayLen(matrix.values)
          ?? arrayLen(data?.values)
          ?? arrayLen(data)
          ?? arrayLen(matrix.rowIndices)
          ?? arrayLen(data?.rowIndices);
      }
    } else {
      return 'Matrix must be an array or object';
    }

    if (!isPositiveInt(rows) || !isPositiveInt(cols)) {
      return 'Matrix dimensions must be positive integers';
    }
    if (rows > max || cols > max) {
      return `Matrix dimension exceeds limit of ${max}`;
    }
    // Bound the actual work/memory, not just each axis.
    if (isDense && rows * cols > this.config.maxElements) {
      return `Matrix element count (${rows} x ${cols}) exceeds limit of ${this.config.maxElements}`;
    }
    if (nnz !== undefined && nnz > this.config.maxNonZeros) {
      return `Matrix non-zero count (${nnz}) exceeds limit of ${this.config.maxNonZeros}`;
    }
    if (!Array.isArray(vector) && !ArrayBuffer.isView(vector)) {
      return 'Vector must be an array';
    }
    if (vector.length !== rows) {
      return `Vector length (${vector.length}) must match matrix rows (${rows})`;
    }
    return null;
  }

  /**
   * Constant-time comparison of a presented token against the configured one,
   * avoiding a timing side-channel on the secret. Returns true when auth is
   * disabled (no configured token).
   */
  tokenMatches(token) {
    const expected = this.config.authToken;
    if (!expected) return true; // auth disabled
    if (typeof token !== 'string' || token.length === 0) return false;
    const a = Buffer.from(token);
    const b = Buffer.from(expected);
    // timingSafeEqual requires equal lengths; length inequality is already a
    // mismatch and the early return leaks only length, not content.
    if (a.length !== b.length) return false;
    return crypto.timingSafeEqual(a, b);
  }

  /**
   * Clamp caller-supplied solver options to safe bounds. An unbounded
   * maxIterations lets one request occupy a worker indefinitely.
   */
  sanitizeOptions(options) {
    const opts = (options && typeof options === 'object') ? { ...options } : {};
    const max = this.config.maxIterations;
    const n = Number(opts.maxIterations);
    if (Number.isFinite(n) && n > 0) {
      opts.maxIterations = Math.min(Math.floor(n), max);
    } else {
      opts.maxIterations = Math.min(1000, max);
    }
    return opts;
  }

  authenticateToken(req, res, next) {
    if (!this.config.authToken) return next(); // auth disabled

    const authHeader = req.headers['authorization'];
    const token = authHeader && authHeader.split(' ')[1];

    if (!token) {
      return res.status(401).json({
        error: 'Authentication token required',
        code: 'AUTH_REQUIRED'
      });
    }

    if (!this.tokenMatches(token)) {
      return res.status(403).json({
        error: 'Invalid authentication token',
        code: 'AUTH_INVALID'
      });
    }

    next();
  }

  async start() {
    return new Promise((resolve, reject) => {
      this.server = this.app.listen(this.config.port, (error) => {
        if (error) {
          reject(error);
          return;
        }

        // Setup WebSocket server
        this.wss = new WebSocketServer({
          server: this.server,
          path: '/ws'
        });

        this.setupWebSocketHandlers();

        console.log(`🚀 Solver server started on port ${this.config.port}`);
        this.emit('started');
        resolve();
      });
    });
  }

  /**
   * Extract a bearer token from a WebSocket upgrade request, accepting either
   * an Authorization header or a `?token=` query parameter (browsers cannot set
   * headers on the WS handshake).
   */
  extractWsToken(req) {
    const authHeader = req.headers['authorization'];
    if (authHeader) return authHeader.split(' ')[1];
    try {
      const url = new URL(req.url, `http://${req.headers.host || 'localhost'}`);
      return url.searchParams.get('token');
    } catch {
      return undefined;
    }
  }

  setupWebSocketHandlers() {
    this.wss.on('connection', (ws, req) => {
      // Enforce the same auth token as the HTTP API. No-op when unset.
      if (this.config.authToken) {
        const token = this.extractWsToken(req);
        if (!this.tokenMatches(token)) {
          ws.send(JSON.stringify({ type: 'error', error: 'Authentication failed', code: 'AUTH_INVALID' }));
          ws.close(1008, 'Authentication failed');
          return;
        }
      }

      console.log('WebSocket connection established');

      ws.on('message', async (data) => {
        try {
          const message = JSON.parse(data.toString());
          await this.handleWebSocketMessage(ws, message);
        } catch (error) {
          console.error('WebSocket message error:', error);
          ws.send(JSON.stringify({
            type: 'error',
            error: error.message
          }));
        }
      });

      ws.on('close', () => {
        console.log('WebSocket connection closed');
      });

      ws.on('error', (error) => {
        console.error('WebSocket error:', error);
      });

      // Send welcome message
      ws.send(JSON.stringify({
        type: 'welcome',
        timestamp: new Date().toISOString()
      }));
    });
  }

  async handleWebSocketMessage(ws, message) {
    switch (message.type) {
      case 'subscribe':
        if (message.session_id) {
          await this.subscribeToSession(ws, message.session_id);
        }
        break;

      case 'solve':
        const sessionId = await this.startWebSocketSolve(ws, message);
        ws.send(JSON.stringify({
          type: 'solve_started',
          session_id: sessionId
        }));
        break;

      case 'ping':
        ws.send(JSON.stringify({
          type: 'pong',
          timestamp: new Date().toISOString()
        }));
        break;

      default:
        ws.send(JSON.stringify({
          type: 'error',
          error: `Unknown message type: ${message.type}`
        }));
    }
  }

  async subscribeToSession(ws, sessionId) {
    const stream = await this.sessions.getJobStream(sessionId);
    if (!stream) {
      ws.send(JSON.stringify({
        type: 'error',
        error: 'Session not found or not streaming'
      }));
      return;
    }

    for await (const update of stream) {
      if (ws.readyState === ws.OPEN) {
        ws.send(JSON.stringify({
          type: 'session_update',
          session_id: sessionId,
          ...update
        }));
      } else {
        break;
      }
    }
  }

  async startWebSocketSolve(ws, message) {
    if (!message.matrix || !message.vector) {
      throw new Error('Matrix and vector are required');
    }
    const sizeError = this.validateProblemSize(message.matrix, message.vector);
    if (sizeError) {
      throw new Error(sizeError);
    }
    const sessionId = uuidv4();
    const session = await this.sessions.createSession(sessionId, {
      matrix: message.matrix,
      vector: message.vector,
      method: message.method || 'adaptive',
      options: this.sanitizeOptions(message.options)
    });

    // Start streaming to WebSocket
    this.subscribeToSession(ws, sessionId);

    return sessionId;
  }

  async stop() {
    return new Promise((resolve) => {
      if (this.wss) {
        this.wss.close();
      }

      if (this.server) {
        this.server.close(() => {
          console.log('Server stopped');
          this.emit('stopped');
          resolve();
        });
      } else {
        resolve();
      }
    });
  }

  getStats() {
    return {
      sessions: this.sessions.getStats(),
      server: {
        uptime: process.uptime(),
        memory: process.memoryUsage(),
        connections: this.wss ? this.wss.clients.size : 0
      }
    };
  }
}

module.exports = { SolverServer };