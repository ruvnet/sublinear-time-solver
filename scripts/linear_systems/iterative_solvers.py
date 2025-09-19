"""
Traditional Iterative Linear System Solvers
Implements classical iterative methods from scratch for detailed analysis.
"""

import numpy as np
import time
from typing import Dict, Any, Optional, Callable, Tuple
import warnings

class IterativeSolvers:
    """Collection of classical iterative methods implemented from scratch."""

    def __init__(self):
        self.convergence_history = []

    def jacobi(self, A: np.ndarray, b: np.ndarray, x0: Optional[np.ndarray] = None,
              max_iter: int = 1000, tol: float = 1e-6,
              callback: Optional[Callable] = None) -> Dict[str, Any]:
        """
        Jacobi iterative method.

        Args:
            A: Coefficient matrix (n x n)
            b: Right-hand side vector (n,)
            x0: Initial guess (if None, use zeros)
            max_iter: Maximum iterations
            tol: Convergence tolerance
            callback: Optional callback function for monitoring

        Returns:
            Dictionary with solution and convergence info
        """
        n = A.shape[0]
        if x0 is None:
            x = np.zeros(n)
        else:
            x = x0.copy()

        # Check diagonal dominance
        diag = np.diag(A)
        if np.any(np.abs(diag) < 1e-12):
            return {
                "solution": None,
                "success": False,
                "error": "Matrix has zero diagonal elements",
                "method": "Jacobi",
                "iterations": 0,
                "time": 0,
                "residual": float('inf')
            }

        start_time = time.perf_counter()
        residuals = []
        x_new = np.zeros_like(x)

        for iteration in range(max_iter):
            # Jacobi iteration: x_i^(k+1) = (b_i - sum(A_ij * x_j^(k))) / A_ii
            for i in range(n):
                sum_ax = 0.0
                for j in range(n):
                    if i != j:
                        sum_ax += A[i, j] * x[j]
                x_new[i] = (b[i] - sum_ax) / A[i, i]

            # Check convergence
            residual = np.linalg.norm(A @ x_new - b)
            residuals.append(residual)

            if callback:
                callback(iteration, x_new, residual)

            if residual < tol:
                solve_time = time.perf_counter() - start_time
                return {
                    "solution": x_new,
                    "success": True,
                    "method": "Jacobi",
                    "iterations": iteration + 1,
                    "time": solve_time,
                    "residual": residual,
                    "convergence_history": residuals,
                    "final_residual": residual,
                    "tolerance": tol
                }

            x, x_new = x_new, x

        # Did not converge
        solve_time = time.perf_counter() - start_time
        final_residual = residuals[-1] if residuals else float('inf')

        return {
            "solution": x,
            "success": False,
            "method": "Jacobi",
            "iterations": max_iter,
            "time": solve_time,
            "residual": final_residual,
            "convergence_history": residuals,
            "final_residual": final_residual,
            "tolerance": tol,
            "error": "Maximum iterations reached"
        }

    def gauss_seidel(self, A: np.ndarray, b: np.ndarray, x0: Optional[np.ndarray] = None,
                     max_iter: int = 1000, tol: float = 1e-6,
                     callback: Optional[Callable] = None) -> Dict[str, Any]:
        """
        Gauss-Seidel iterative method.

        Args:
            A: Coefficient matrix (n x n)
            b: Right-hand side vector (n,)
            x0: Initial guess (if None, use zeros)
            max_iter: Maximum iterations
            tol: Convergence tolerance
            callback: Optional callback function for monitoring

        Returns:
            Dictionary with solution and convergence info
        """
        n = A.shape[0]
        if x0 is None:
            x = np.zeros(n)
        else:
            x = x0.copy()

        # Check diagonal elements
        diag = np.diag(A)
        if np.any(np.abs(diag) < 1e-12):
            return {
                "solution": None,
                "success": False,
                "error": "Matrix has zero diagonal elements",
                "method": "Gauss-Seidel",
                "iterations": 0,
                "time": 0,
                "residual": float('inf')
            }

        start_time = time.perf_counter()
        residuals = []

        for iteration in range(max_iter):
            x_old = x.copy()

            # Gauss-Seidel iteration: use updated values immediately
            for i in range(n):
                sum_ax = 0.0
                for j in range(i):
                    sum_ax += A[i, j] * x[j]  # Updated values
                for j in range(i + 1, n):
                    sum_ax += A[i, j] * x[j]  # Old values
                x[i] = (b[i] - sum_ax) / A[i, i]

            # Check convergence
            residual = np.linalg.norm(A @ x - b)
            residuals.append(residual)

            if callback:
                callback(iteration, x, residual)

            if residual < tol:
                solve_time = time.perf_counter() - start_time
                return {
                    "solution": x,
                    "success": True,
                    "method": "Gauss-Seidel",
                    "iterations": iteration + 1,
                    "time": solve_time,
                    "residual": residual,
                    "convergence_history": residuals,
                    "final_residual": residual,
                    "tolerance": tol
                }

        # Did not converge
        solve_time = time.perf_counter() - start_time
        final_residual = residuals[-1] if residuals else float('inf')

        return {
            "solution": x,
            "success": False,
            "method": "Gauss-Seidel",
            "iterations": max_iter,
            "time": solve_time,
            "residual": final_residual,
            "convergence_history": residuals,
            "final_residual": final_residual,
            "tolerance": tol,
            "error": "Maximum iterations reached"
        }

    def sor(self, A: np.ndarray, b: np.ndarray, omega: float = 1.0,
            x0: Optional[np.ndarray] = None, max_iter: int = 1000,
            tol: float = 1e-6, callback: Optional[Callable] = None) -> Dict[str, Any]:
        """
        Successive Over-Relaxation (SOR) method.

        Args:
            A: Coefficient matrix (n x n)
            b: Right-hand side vector (n,)
            omega: Relaxation parameter (1.0 = Gauss-Seidel)
            x0: Initial guess (if None, use zeros)
            max_iter: Maximum iterations
            tol: Convergence tolerance
            callback: Optional callback function for monitoring

        Returns:
            Dictionary with solution and convergence info
        """
        n = A.shape[0]
        if x0 is None:
            x = np.zeros(n)
        else:
            x = x0.copy()

        # Check diagonal elements
        diag = np.diag(A)
        if np.any(np.abs(diag) < 1e-12):
            return {
                "solution": None,
                "success": False,
                "error": "Matrix has zero diagonal elements",
                "method": f"SOR(ω={omega})",
                "iterations": 0,
                "time": 0,
                "residual": float('inf')
            }

        start_time = time.perf_counter()
        residuals = []

        for iteration in range(max_iter):
            x_old = x.copy()

            # SOR iteration
            for i in range(n):
                sum_ax = 0.0
                for j in range(i):
                    sum_ax += A[i, j] * x[j]  # Updated values
                for j in range(i + 1, n):
                    sum_ax += A[i, j] * x[j]  # Old values

                x_gs = (b[i] - sum_ax) / A[i, i]  # Gauss-Seidel step
                x[i] = (1 - omega) * x[i] + omega * x_gs  # SOR step

            # Check convergence
            residual = np.linalg.norm(A @ x - b)
            residuals.append(residual)

            if callback:
                callback(iteration, x, residual)

            if residual < tol:
                solve_time = time.perf_counter() - start_time
                return {
                    "solution": x,
                    "success": True,
                    "method": f"SOR(ω={omega})",
                    "iterations": iteration + 1,
                    "time": solve_time,
                    "residual": residual,
                    "convergence_history": residuals,
                    "final_residual": residual,
                    "tolerance": tol,
                    "omega": omega
                }

        # Did not converge
        solve_time = time.perf_counter() - start_time
        final_residual = residuals[-1] if residuals else float('inf')

        return {
            "solution": x,
            "success": False,
            "method": f"SOR(ω={omega})",
            "iterations": max_iter,
            "time": solve_time,
            "residual": final_residual,
            "convergence_history": residuals,
            "final_residual": final_residual,
            "tolerance": tol,
            "omega": omega,
            "error": "Maximum iterations reached"
        }

    def conjugate_gradient(self, A: np.ndarray, b: np.ndarray,
                          x0: Optional[np.ndarray] = None, max_iter: int = 1000,
                          tol: float = 1e-6, callback: Optional[Callable] = None) -> Dict[str, Any]:
        """
        Conjugate Gradient method (for symmetric positive definite matrices).

        Args:
            A: Coefficient matrix (n x n, must be symmetric positive definite)
            b: Right-hand side vector (n,)
            x0: Initial guess (if None, use zeros)
            max_iter: Maximum iterations
            tol: Convergence tolerance
            callback: Optional callback function for monitoring

        Returns:
            Dictionary with solution and convergence info
        """
        n = A.shape[0]
        if x0 is None:
            x = np.zeros(n)
        else:
            x = x0.copy()

        # Check if matrix is symmetric
        if not np.allclose(A, A.T):
            warnings.warn("Matrix is not symmetric, CG may not converge")

        start_time = time.perf_counter()
        residuals = []

        # Initial residual
        r = b - A @ x
        p = r.copy()
        rsold = r.T @ r

        for iteration in range(max_iter):
            Ap = A @ p
            alpha = rsold / (p.T @ Ap)
            x = x + alpha * p
            r = r - alpha * Ap
            rsnew = r.T @ r

            residual = np.sqrt(rsnew)
            residuals.append(residual)

            if callback:
                callback(iteration, x, residual)

            if residual < tol:
                solve_time = time.perf_counter() - start_time
                return {
                    "solution": x,
                    "success": True,
                    "method": "Conjugate Gradient",
                    "iterations": iteration + 1,
                    "time": solve_time,
                    "residual": residual,
                    "convergence_history": residuals,
                    "final_residual": residual,
                    "tolerance": tol
                }

            beta = rsnew / rsold
            p = r + beta * p
            rsold = rsnew

        # Did not converge
        solve_time = time.perf_counter() - start_time
        final_residual = residuals[-1] if residuals else float('inf')

        return {
            "solution": x,
            "success": False,
            "method": "Conjugate Gradient",
            "iterations": max_iter,
            "time": solve_time,
            "residual": final_residual,
            "convergence_history": residuals,
            "final_residual": final_residual,
            "tolerance": tol,
            "error": "Maximum iterations reached"
        }

    def analyze_convergence(self, A: np.ndarray) -> Dict[str, Any]:
        """
        Analyze convergence properties of iterative methods.

        Args:
            A: Coefficient matrix

        Returns:
            Dictionary with convergence analysis
        """
        n = A.shape[0]
        analysis = {}

        try:
            # Jacobi iteration matrix: T_J = -D^(-1)(L + U)
            D = np.diag(np.diag(A))
            L = np.tril(A, -1)
            U = np.triu(A, 1)

            if np.any(np.abs(np.diag(D)) < 1e-12):
                return {"error": "Matrix has zero diagonal elements"}

            D_inv = np.diag(1.0 / np.diag(A))
            T_jacobi = -D_inv @ (L + U)

            # Gauss-Seidel iteration matrix: T_GS = -(D + L)^(-1) U
            try:
                DL_inv = np.linalg.inv(D + L)
                T_gauss_seidel = -DL_inv @ U
            except:
                T_gauss_seidel = None

            # Spectral radius (convergence condition)
            jacobi_eigenvals = np.linalg.eigvals(T_jacobi)
            jacobi_spectral_radius = np.max(np.abs(jacobi_eigenvals))

            analysis["jacobi"] = {
                "spectral_radius": jacobi_spectral_radius,
                "convergent": jacobi_spectral_radius < 1.0,
                "iteration_matrix_norm": np.linalg.norm(T_jacobi),
                "eigenvalues": jacobi_eigenvals
            }

            if T_gauss_seidel is not None:
                gs_eigenvals = np.linalg.eigvals(T_gauss_seidel)
                gs_spectral_radius = np.max(np.abs(gs_eigenvals))

                analysis["gauss_seidel"] = {
                    "spectral_radius": gs_spectral_radius,
                    "convergent": gs_spectral_radius < 1.0,
                    "iteration_matrix_norm": np.linalg.norm(T_gauss_seidel),
                    "eigenvalues": gs_eigenvals
                }

            # Diagonal dominance check
            diag = np.abs(np.diag(A))
            off_diag_sum = np.sum(np.abs(A), axis=1) - diag
            analysis["diagonal_dominance"] = {
                "strict": np.all(diag > off_diag_sum),
                "weak": np.all(diag >= off_diag_sum),
                "ratios": diag / (off_diag_sum + 1e-12)
            }

            # CG convergence (for symmetric matrices)
            if np.allclose(A, A.T):
                eigenvals = np.linalg.eigvals(A)
                condition_number = np.max(eigenvals) / np.min(eigenvals)
                analysis["conjugate_gradient"] = {
                    "applicable": True,
                    "condition_number": condition_number,
                    "eigenvalues": eigenvals,
                    "positive_definite": np.all(eigenvals > 0)
                }
            else:
                analysis["conjugate_gradient"] = {
                    "applicable": False,
                    "reason": "Matrix not symmetric"
                }

        except Exception as e:
            analysis["error"] = f"Analysis failed: {str(e)}"

        return analysis

    def benchmark_all_iterative(self, A: np.ndarray, b: np.ndarray,
                               max_iter: int = 1000, tol: float = 1e-6) -> Dict[str, Dict[str, Any]]:
        """
        Benchmark all iterative methods.

        Args:
            A: Coefficient matrix
            b: Right-hand side vector
            max_iter: Maximum iterations for all methods
            tol: Convergence tolerance

        Returns:
            Dictionary mapping method names to results
        """
        results = {}

        # Basic methods
        results["jacobi"] = self.jacobi(A, b, max_iter=max_iter, tol=tol)
        results["gauss_seidel"] = self.gauss_seidel(A, b, max_iter=max_iter, tol=tol)

        # SOR with different omega values
        omega_values = [1.2, 1.5, 1.8]
        for omega in omega_values:
            results[f"sor_{omega}"] = self.sor(A, b, omega=omega, max_iter=max_iter, tol=tol)

        # Conjugate Gradient (if applicable)
        if np.allclose(A, A.T):
            results["conjugate_gradient"] = self.conjugate_gradient(A, b, max_iter=max_iter, tol=tol)

        # Add convergence analysis
        results["convergence_analysis"] = self.analyze_convergence(A)

        return results

def optimal_sor_parameter(A: np.ndarray) -> float:
    """
    Estimate optimal SOR parameter for convergence.

    Args:
        A: Coefficient matrix

    Returns:
        Estimated optimal omega value
    """
    try:
        n = A.shape[0]
        D = np.diag(np.diag(A))
        L = np.tril(A, -1)
        U = np.triu(A, 1)

        if np.any(np.abs(np.diag(D)) < 1e-12):
            return 1.0  # Default to Gauss-Seidel

        # Jacobi iteration matrix
        D_inv = np.diag(1.0 / np.diag(A))
        T_jacobi = -D_inv @ (L + U)

        # Spectral radius of Jacobi method
        jacobi_eigenvals = np.linalg.eigvals(T_jacobi)
        rho_jacobi = np.max(np.abs(jacobi_eigenvals))

        if rho_jacobi >= 1.0:
            return 1.0  # Jacobi doesn't converge

        # Optimal omega for symmetric matrix
        if np.allclose(A, A.T):
            omega_opt = 2.0 / (1.0 + np.sqrt(1.0 - rho_jacobi**2))
        else:
            # Approximation for non-symmetric case
            omega_opt = 2.0 / (1.0 + rho_jacobi)

        # Clamp to reasonable range
        omega_opt = max(1.0, min(2.0, omega_opt))

        return omega_opt

    except:
        return 1.0  # Default to Gauss-Seidel if estimation fails