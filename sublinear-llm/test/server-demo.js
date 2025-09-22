#!/usr/bin/env node
/**
 * HTTP Server Demo - OpenAI Compatible API Server
 * Provides REST endpoints that match OpenAI's API exactly
 */

import { readFileSync } from 'fs';
import { fileURLToPath } from 'url';
import { dirname, join } from 'path';
import { createServer } from 'http';
import { parse } from 'url';

const __filename = fileURLToPath(import.meta.url);
const __dirname = dirname(__filename);

const wasmPath = join(__dirname, '../pkg/sublinear_llm_bg.wasm');
const wasmBytes = readFileSync(wasmPath);

// Initialize LLM
let llm = null;

async function initLLM() {
    const module = await import('../pkg/sublinear_llm.js');
    await module.default(wasmBytes);
    llm = new module.SublinearLLM();
    console.log('✅ Sublinear LLM initialized');
}

// CORS headers
function setCORSHeaders(res) {
    res.setHeader('Access-Control-Allow-Origin', '*');
    res.setHeader('Access-Control-Allow-Methods', 'GET, POST, OPTIONS');
    res.setHeader('Access-Control-Allow-Headers', 'Content-Type, Authorization');
}

// Parse request body
function parseBody(req) {
    return new Promise((resolve, reject) => {
        let body = '';
        req.on('data', chunk => body += chunk);
        req.on('end', () => {
            try {
                resolve(body ? JSON.parse(body) : {});
            } catch (e) {
                reject(e);
            }
        });
    });
}

// HTTP Server
const server = createServer(async (req, res) => {
    setCORSHeaders(res);

    if (req.method === 'OPTIONS') {
        res.writeHead(200);
        res.end();
        return;
    }

    const { pathname } = parse(req.url);

    try {
        if (pathname === '/v1/chat/completions' && req.method === 'POST') {
            const body = await parseBody(req);
            console.log('📨 Chat Completions Request:', JSON.stringify(body, null, 2));

            const response = llm.chatCompletions(JSON.stringify(body));
            const parsedResponse = JSON.parse(response);

            if (body.stream) {
                // Streaming response
                res.writeHead(200, {
                    'Content-Type': 'text/event-stream',
                    'Cache-Control': 'no-cache',
                    'Connection': 'keep-alive'
                });
                res.write(response);
                res.end();
            } else {
                // Regular response
                res.writeHead(200, { 'Content-Type': 'application/json' });
                res.end(JSON.stringify(parsedResponse, null, 2));
            }

        } else if (pathname === '/v1/completions' && req.method === 'POST') {
            const body = await parseBody(req);
            console.log('📨 Completions Request:', JSON.stringify(body, null, 2));

            const response = llm.completions(JSON.stringify(body));
            res.writeHead(200, { 'Content-Type': 'application/json' });
            res.end(response);

        } else if (pathname === '/v1/responses' && req.method === 'POST') {
            const body = await parseBody(req);
            console.log('📨 O1 Responses Request:', JSON.stringify(body, null, 2));

            const response = llm.responsesAPI(JSON.stringify(body));
            const parsedResponse = JSON.parse(response);

            if (body.stream) {
                // Streaming response
                res.writeHead(200, {
                    'Content-Type': 'text/event-stream',
                    'Cache-Control': 'no-cache',
                    'Connection': 'keep-alive'
                });
                res.write(response);
                res.end();
            } else {
                // Regular response
                res.writeHead(200, { 'Content-Type': 'application/json' });
                res.end(JSON.stringify(parsedResponse, null, 2));
            }

        } else if (pathname === '/health' && req.method === 'GET') {
            const health = llm.healthCheck();
            res.writeHead(200, { 'Content-Type': 'application/json' });
            res.end(health);

        } else if (pathname === '/stats' && req.method === 'GET') {
            const stats = llm.trainingStats();
            res.writeHead(200, { 'Content-Type': 'application/json' });
            res.end(stats);

        } else if (pathname === '/train' && req.method === 'POST') {
            const body = await parseBody(req);
            console.log('📚 Training Request:', JSON.stringify(body, null, 2));

            const result = llm.train(JSON.stringify(body));
            res.writeHead(200, { 'Content-Type': 'application/json' });
            res.end(result);

        } else if (pathname === '/' && req.method === 'GET') {
            // Serve HTML demo
            const htmlPath = join(__dirname, 'web-demo.html');
            const html = readFileSync(htmlPath, 'utf-8');
            res.writeHead(200, { 'Content-Type': 'text/html' });
            res.end(html);

        } else if (pathname.startsWith('/pkg/') && req.method === 'GET') {
            // Serve WASM package files
            try {
                const filePath = join(__dirname, '..', pathname);
                const fileContent = readFileSync(filePath);

                let contentType = 'application/octet-stream';
                if (pathname.endsWith('.js')) contentType = 'application/javascript';
                else if (pathname.endsWith('.wasm')) contentType = 'application/wasm';
                else if (pathname.endsWith('.d.ts')) contentType = 'text/plain';

                res.writeHead(200, { 'Content-Type': contentType });
                res.end(fileContent);
            } catch (error) {
                res.writeHead(404, { 'Content-Type': 'application/json' });
                res.end(JSON.stringify({ error: 'File not found' }));
            }

        } else {
            res.writeHead(404, { 'Content-Type': 'application/json' });
            res.end(JSON.stringify({ error: 'Endpoint not found' }));
        }
    } catch (error) {
        console.error('❌ Server Error:', error);
        res.writeHead(500, { 'Content-Type': 'application/json' });
        res.end(JSON.stringify({ error: error.message }));
    }
});

// Start server
const PORT = process.env.PORT || 3000;

initLLM().then(() => {
    server.listen(PORT, () => {
        console.log('\n🚀 SUBLINEAR LLM API SERVER RUNNING');
        console.log('='.repeat(50));
        console.log(`🌐 Server: http://localhost:${PORT}`);
        console.log(`📱 Web Demo: http://localhost:${PORT}/`);
        console.log('\n🔗 API Endpoints:');
        console.log(`  POST /v1/chat/completions     - GPT-5 Style Chat`);
        console.log(`  POST /v1/completions          - Legacy Completions`);
        console.log(`  POST /v1/responses            - O1 Style Reasoning`);
        console.log(`  POST /train                   - Enhanced Training`);
        console.log(`  GET  /health                  - System Health`);
        console.log(`  GET  /stats                   - Training Stats`);
        console.log('\n💡 Test with curl:');
        console.log(`curl -X POST http://localhost:${PORT}/v1/chat/completions \\`);
        console.log(`  -H "Content-Type: application/json" \\`);
        console.log(`  -d '{"model":"sublinear-gpt5","messages":[{"role":"user","content":"Hello!"}]}'`);
        console.log('\n🎯 Ready for GPT-5 style reasoning!');
    });
}).catch(error => {
    console.error('❌ Failed to start server:', error);
    process.exit(1);
});