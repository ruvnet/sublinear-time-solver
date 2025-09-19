const http = require('http');

console.log('=== API Validation Test ===\n');

// Test data
const testSystem = {
  matrix: [[4, -1, 0], [-1, 4, -1], [0, -1, 3]],
  vector: [15, 10, 10],
  options: {
    method: 'jacobi',
    tolerance: 1e-6,
    maxIterations: 100
  }
};

// Start server in background
const { spawn } = require('child_process');
const server = spawn('node', ['../../bin/cli.js', 'serve', '--port', '3003'], {
  detached: false,
  stdio: 'pipe'
});

server.stderr.on('data', (data) => {
  console.log(`Server: ${data}`);
});

// Wait for server to start
setTimeout(() => {
  const postData = JSON.stringify(testSystem);
  
  const options = {
    hostname: 'localhost',
    port: 3003,
    path: '/solve',
    method: 'POST',
    headers: {
      'Content-Type': 'application/json',
      'Content-Length': Buffer.byteLength(postData)
    }
  };
  
  const req = http.request(options, (res) => {
    console.log(`Status: ${res.statusCode}`);
    console.log(`Headers: ${JSON.stringify(res.headers)}`);
    
    let data = '';
    res.on('data', (chunk) => {
      data += chunk;
    });
    
    res.on('end', () => {
      console.log('Response:', data.substring(0, 200));
      server.kill();
      process.exit(0);
    });
  });
  
  req.on('error', (e) => {
    console.error(`Problem with request: ${e.message}`);
    server.kill();
    process.exit(1);
  });
  
  req.write(postData);
  req.end();
}, 2000);

// Timeout safety
setTimeout(() => {
  console.log('Test timeout');
  server.kill();
  process.exit(1);
}, 10000);
