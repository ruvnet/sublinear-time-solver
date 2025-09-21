# Module Not Found Error: @psycho-symbolic-reasoner Remote Installation

## Bug Description
When attempting to install and run `@psycho-symbolic-reasoner` as a standalone package on a remote system, the installation fails with a MODULE_NOT_FOUND error pointing to a missing dist/mcp/server.js file.

## Error Message
```
Error: Cannot find module '/Users/NoMind/Code/sublinear-time-solver/psycho-symbolic-reasoner/dist/mcp/server.js'
    at Module._resolveFilename (node:internal/modules/cjs/loader:1420:15)
    at defaultResolveImpl (node:internal/modules/cjs/loader:1058:19)
    at resolveForCJSWithHooks (node:internal/modules/cjs/loader:1063:22)
    at Module._load (node:internal/modules/cjs/loader:1226:37)
    at TracingChannel.traceSync (node:diagnostics_channel:322:14)
    at wrapModuleLoad (node:internal/modules/cjs/loader:244:24)
    at Module.executeUserEntryPoint [as runMain] (node:internal/modules/run_main:154:5)
    at node:internal/main/run_main_module:33:47 {
  code: 'MODULE_NOT_FOUND',
  requireStack: []
}
```

## Environment
- **OS**: macOS (based on /Users path)
- **Node Version**: Latest (based on error stack trace format)
- **Package**: @psycho-symbolic-reasoner
- **Installation Method**: npm install

## Root Cause Analysis
After investigating the codebase structure, the issue appears to be:

1. **Missing Standalone Package**: The psycho-symbolic-reasoner functionality is integrated into the main `sublinear-time-solver` package but is being referenced as if it were a separate npm package `@psycho-symbolic-reasoner`.

2. **Incorrect Path Resolution**: The error shows it's looking for files in a non-existent directory structure:
   - Expected: `/Users/NoMind/Code/sublinear-time-solver/psycho-symbolic-reasoner/dist/mcp/server.js`
   - Actual structure: The MCP server is at `dist/mcp/server.js` in the main package

3. **Package Configuration Issue**: The `package.json` doesn't define `@psycho-symbolic-reasoner` as a separate package, and there's no npm package published under this name.

## Steps to Reproduce
1. Attempt to install `@psycho-symbolic-reasoner` as a standalone package:
   ```bash
   npm install @psycho-symbolic-reasoner
   ```
2. Try to run the package:
   ```bash
   npm start
   ```
3. Observe the MODULE_NOT_FOUND error

## Expected Behavior
Either:
- The psycho-symbolic-reasoner should be available as a standalone npm package, OR
- Clear documentation should indicate it's part of the main `sublinear-time-solver` package

## Actual Behavior
The package installation fails with MODULE_NOT_FOUND error, preventing usage of the psycho-symbolic reasoning features.

## Proposed Solutions

### Option 1: Publish as Standalone Package
Create and publish `@psycho-symbolic-reasoner` as a separate npm package with:
- Independent package.json
- Proper entry points
- Bundled dependencies
- Clear installation instructions

### Option 2: Document as Integrated Feature
Update documentation to clarify that psycho-symbolic-reasoner is accessed through the main package:
```bash
npm install sublinear-time-solver
# Access via MCP tools or CLI
npx sublinear-solver serve
```

### Option 3: Create Package Alias
Add an export alias in the main package.json:
```json
"exports": {
  "./psycho-symbolic": {
    "import": "./dist/mcp/tools/psycho-symbolic.js",
    "types": "./dist/mcp/tools/psycho-symbolic.d.ts"
  }
}
```

## Temporary Workaround
Users experiencing this issue can use the main package instead:
```bash
# Install the main package
npm install sublinear-time-solver

# Build the package
npm run build

# Start the MCP server
npm start
```

## Additional Context
- The psycho-symbolic-reasoner is documented as a Claude Code agent in `.claude/agents/psycho-symbolic-reasoner/`
- The functionality exists in `src/mcp/tools/psycho-symbolic.ts`
- The main package version is 1.0.3
- The package uses ES modules (`"type": "module"`)

## Labels
- bug
- installation
- packaging
- documentation

## Priority
High - Blocks users from accessing psycho-symbolic reasoning features

## Related Files
- `/src/mcp/tools/psycho-symbolic.ts`
- `/src/reasongraph/`
- `/.claude/agents/psycho-symbolic-reasoner/psycho-symbolic-reasoner.md`
- `/package.json`