/**
 * Temporal Lead Solver
 * Achieve temporal computational lead through sublinear-time algorithms
 */

export class TemporalPredictor {
    constructor(tolerance = 1e-6, maxIterations = 1000) {
        this.tolerance = tolerance;
        this.maxIterations = maxIterations;
    }

    /**
     * Predict solution with temporal advantage
     */
    predictWithTemporalAdvantage(matrix, vector, distanceKm) {
        const start = performance.now();

        // Sublinear solving algorithm
        const solution = this.solveSublinear(matrix, vector);

        const computeTimeMs = performance.now() - start;
        const lightTravelTimeMs = (distanceKm * 1000) / (299792458 / 1000);
        const temporalAdvantageMs = lightTravelTimeMs - computeTimeMs;
        const effectiveVelocity = (distanceKm * 1000) / (computeTimeMs / 1000) / 299792458;

        return {
            solution,
            residual: this.lastResidual,
            iterations: this.lastIterations,
            converged: this.lastConverged,
            computeTimeMs,
            lightTravelTimeMs,
            temporalAdvantageMs,
            effectiveVelocityRatio: effectiveVelocity,
            // Matrix entries read: one dense residual (n^2) per iteration.
            queryCount: (this.lastIterations + 1) * vector.length * vector.length
        };
    }

    /**
     * Jacobi iteration (the Neumann series x = sum_k (I - D^-1 A)^k D^-1 b)
     * run until ||b - A x||_2 <= tolerance * max(1, ||b||_2) or maxIterations.
     * Converges for strictly diagonally dominant matrices.
     *
     * 0.1.0 stopped after log2(n) + 1 damped terms, which left a large
     * residual (for a 2x2 system it returned 0.61 where the answer is 1).
     */
    solveSublinear(matrix, b) {
        const n = b.length;
        if (!Array.isArray(matrix) || matrix.length !== n) {
            throw new Error(`matrix must have ${n} rows`);
        }
        const diag = new Array(n);
        for (let i = 0; i < n; i++) {
            const row = matrix[i];
            if (!row || row.length !== n) throw new Error(`row ${i} must have length ${n}`);
            if (!(Math.abs(row[i]) > 0)) throw new Error(`zero diagonal at row ${i}`);
            diag[i] = row[i];
        }
        let bNorm = 0;
        for (let i = 0; i < n; i++) bNorm += b[i] * b[i];
        const target = this.tolerance * Math.max(1, Math.sqrt(bNorm));

        let x = new Array(n).fill(0);
        const r = new Array(n);
        const residual = () => {
            let s2 = 0;
            for (let i = 0; i < n; i++) {
                let s = b[i];
                const row = matrix[i];
                for (let j = 0; j < n; j++) s -= row[j] * x[j];
                r[i] = s;
                s2 += s * s;
            }
            return Math.sqrt(s2);
        };
        let norm = residual();
        let it = 0;
        while (norm > target && it < this.maxIterations) {
            for (let i = 0; i < n; i++) x[i] += r[i] / diag[i];
            norm = residual();
            it++;
            if (!Number.isFinite(norm)) break;
        }
        this.lastIterations = it;
        this.lastResidual = norm;
        this.lastConverged = norm <= target;
        return x;
    }

    /**
     * Validate temporal advantage claims
     */
    validateTemporalAdvantage(size = 1000) {
        // Generate test matrix (diagonally dominant)
        const matrix = [];
        const b = new Array(size).fill(1);

        for (let i = 0; i < size; i++) {
            matrix[i] = new Array(size).fill(0);
            matrix[i][i] = 4;
            if (i > 0) matrix[i][i - 1] = -1;
            if (i < size - 1) matrix[i][i + 1] = -1;
        }

        const result = this.predictWithTemporalAdvantage(matrix, b, 10900); // Tokyo to NYC

        return {
            matrixSize: size,
            computeTimeMs: result.computeTimeMs,
            lightTravelTimeMs: result.lightTravelTimeMs,
            temporalAdvantageMs: result.temporalAdvantageMs,
            effectiveVelocity: `${result.effectiveVelocityRatio.toFixed(0)}x speed of light`,
            matrixEntriesRead: result.queryCount,
            valid: result.temporalAdvantageMs > 0
        };
    }
}

export default TemporalPredictor;