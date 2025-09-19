"""
Traditional Linear System Solvers
Implements direct and iterative methods using NumPy and SciPy for comparison.
"""

import numpy as np
import scipy.sparse as sp
import scipy.sparse.linalg as spla
import scipy.linalg as la
import time
from typing import Dict, Any, Tuple, Optional
import warnings

class TraditionalSolvers:
    """Collection of traditional linear system solving methods."""

    def __init__(self):
        self.results = {}

    def direct_numpy(self, A: np.ndarray, b: np.ndarray,
                    method: str = "solve") -> Dict[str, Any]:
        """
        Direct methods using NumPy.

        Args:
            A: Coefficient matrix (n x n)
            b: Right-hand side vector (n,)
            method: 'solve', 'lu', 'cholesky', 'qr'

        Returns:
            Dictionary with solution, time, and metadata
        """
        start_time = time.perf_counter()

        try:
            if method == "solve":
                # General direct solver
                x = np.linalg.solve(A, b)
                algorithm = "LU decomposition (LAPACK)"

            elif method == "lu":
                # Explicit LU decomposition
                P, L, U = la.lu(A)
                y = la.solve_triangular(L, P.T @ b, lower=True)
                x = la.solve_triangular(U, y, lower=False)
                algorithm = "Explicit LU decomposition"

            elif method == "cholesky":
                # Cholesky for symmetric positive definite
                if not np.allclose(A, A.T):
                    raise ValueError("Matrix must be symmetric for Cholesky")
                L = la.cholesky(A, lower=True)
                y = la.solve_triangular(L, b, lower=True)
                x = la.solve_triangular(L.T, y, lower=False)
                algorithm = "Cholesky decomposition"

            elif method == "qr":
                # QR decomposition
                Q, R = la.qr(A)
                x = la.solve_triangular(R, Q.T @ b, lower=False)
                algorithm = "QR decomposition"

            else:
                raise ValueError(f"Unknown method: {method}")

            solve_time = time.perf_counter() - start_time

            # Compute residual
            residual = np.linalg.norm(A @ x - b)

            return {
                "solution": x,
                "time": solve_time,
                "residual": residual,
                "method": f"NumPy {method}",
                "algorithm": algorithm,
                "success": True,
                "iterations": 1,  # Direct methods are single-step
                "memory_efficient": False
            }

        except Exception as e:
            return {
                "solution": None,
                "time": time.perf_counter() - start_time,
                "residual": float('inf'),
                "method": f"NumPy {method}",
                "algorithm": algorithm if 'algorithm' in locals() else "Unknown",
                "success": False,
                "error": str(e),
                "iterations": 0,
                "memory_efficient": False
            }

    def sparse_scipy(self, A: sp.spmatrix, b: np.ndarray,
                    method: str = "spsolve") -> Dict[str, Any]:
        """
        Sparse direct methods using SciPy.

        Args:
            A: Sparse coefficient matrix
            b: Right-hand side vector
            method: 'spsolve', 'umfpack', 'superlu'

        Returns:
            Dictionary with solution, time, and metadata
        """
        start_time = time.perf_counter()

        try:
            if method == "spsolve":
                # Default sparse solver
                x = spla.spsolve(A, b)
                algorithm = "SuperLU (default)"

            elif method == "umfpack":
                # UMFPACK if available
                try:
                    from scikits.umfpack import spsolve as umfpack_solve
                    x = umfpack_solve(A, b)
                    algorithm = "UMFPACK"
                except ImportError:
                    # Fallback to default
                    x = spla.spsolve(A, b, use_umfpack=False)
                    algorithm = "SuperLU (UMFPACK unavailable)"

            elif method == "superlu":
                # Explicit SuperLU
                x = spla.spsolve(A, b, use_umfpack=False)
                algorithm = "SuperLU"

            else:
                raise ValueError(f"Unknown sparse method: {method}")

            solve_time = time.perf_counter() - start_time

            # Compute residual
            residual = np.linalg.norm(A @ x - b)

            return {
                "solution": x,
                "time": solve_time,
                "residual": residual,
                "method": f"SciPy sparse {method}",
                "algorithm": algorithm,
                "success": True,
                "iterations": 1,
                "memory_efficient": True
            }

        except Exception as e:
            return {
                "solution": None,
                "time": time.perf_counter() - start_time,
                "residual": float('inf'),
                "method": f"SciPy sparse {method}",
                "algorithm": algorithm if 'algorithm' in locals() else "Unknown",
                "success": False,
                "error": str(e),
                "iterations": 0,
                "memory_efficient": True
            }

    def iterative_scipy(self, A, b: np.ndarray, method: str = "gmres",
                       tol: float = 1e-6, maxiter: int = 1000) -> Dict[str, Any]:
        """
        Iterative methods using SciPy.

        Args:
            A: Coefficient matrix (dense or sparse)
            b: Right-hand side vector
            method: 'gmres', 'bicgstab', 'cg', 'minres'
            tol: Convergence tolerance
            maxiter: Maximum iterations

        Returns:
            Dictionary with solution, time, and metadata
        """
        start_time = time.perf_counter()

        # Convert to sparse if dense and large
        if isinstance(A, np.ndarray) and A.shape[0] > 1000:
            A = sp.csr_matrix(A)

        try:
            if method == "gmres":
                x, info = spla.gmres(A, b, tol=tol, maxiter=maxiter)
                algorithm = "GMRES (Generalized Minimal Residual)"

            elif method == "bicgstab":
                x, info = spla.bicgstab(A, b, tol=tol, maxiter=maxiter)
                algorithm = "BiCGSTAB (Biconjugate Gradient Stabilized)"

            elif method == "cg":
                x, info = spla.cg(A, b, tol=tol, maxiter=maxiter)
                algorithm = "Conjugate Gradient"

            elif method == "minres":
                x, info = spla.minres(A, b, tol=tol, maxiter=maxiter)
                algorithm = "MINRES (Minimal Residual)"

            else:
                raise ValueError(f"Unknown iterative method: {method}")

            solve_time = time.perf_counter() - start_time

            # Check convergence
            success = (info == 0)
            if not success:
                warnings.warn(f"Iterative solver did not converge (info={info})")

            # Compute residual
            residual = np.linalg.norm(A @ x - b) if x is not None else float('inf')

            return {
                "solution": x,
                "time": solve_time,
                "residual": residual,
                "method": f"SciPy {method}",
                "algorithm": algorithm,
                "success": success,
                "convergence_info": info,
                "iterations": maxiter if not success else "unknown",
                "memory_efficient": True,
                "tolerance": tol
            }

        except Exception as e:
            return {
                "solution": None,
                "time": time.perf_counter() - start_time,
                "residual": float('inf'),
                "method": f"SciPy {method}",
                "algorithm": algorithm if 'algorithm' in locals() else "Unknown",
                "success": False,
                "error": str(e),
                "iterations": 0,
                "memory_efficient": True,
                "tolerance": tol
            }

    def benchmark_all_methods(self, A, b: np.ndarray,
                             sparse_threshold: int = 1000) -> Dict[str, Dict[str, Any]]:
        """
        Benchmark all available traditional methods.

        Args:
            A: Coefficient matrix
            b: Right-hand side vector
            sparse_threshold: Size threshold for using sparse methods

        Returns:
            Dictionary mapping method names to results
        """
        results = {}
        n = A.shape[0]

        # Determine if matrix should be treated as sparse
        is_large = n >= sparse_threshold

        # Convert to appropriate format
        if isinstance(A, np.ndarray):
            A_dense = A
            A_sparse = sp.csr_matrix(A)
        else:
            A_sparse = A
            A_dense = A.toarray() if n < sparse_threshold else None

        # Direct methods for smaller dense matrices
        if A_dense is not None and n < sparse_threshold:
            methods = ["solve", "lu", "qr"]

            # Add Cholesky if symmetric positive definite
            if np.allclose(A_dense, A_dense.T):
                try:
                    np.linalg.cholesky(A_dense)
                    methods.append("cholesky")
                except:
                    pass  # Not positive definite

            for method in methods:
                results[f"numpy_{method}"] = self.direct_numpy(A_dense, b, method)

        # Sparse direct methods
        sparse_methods = ["spsolve", "superlu"]
        for method in sparse_methods:
            results[f"scipy_sparse_{method}"] = self.sparse_scipy(A_sparse, b, method)

        # Iterative methods
        iterative_methods = ["gmres", "bicgstab"]

        # Add CG for symmetric matrices
        if isinstance(A, np.ndarray):
            is_symmetric = np.allclose(A, A.T)
        else:
            # Check symmetry for sparse matrix (approximation)
            is_symmetric = np.allclose((A - A.T).data, 0) if hasattr(A, 'data') else False

        if is_symmetric:
            iterative_methods.extend(["cg", "minres"])

        for method in iterative_methods:
            results[f"scipy_iter_{method}"] = self.iterative_scipy(A, b, method)

        return results

def analyze_matrix_properties(A) -> Dict[str, Any]:
    """
    Analyze matrix properties relevant for solver selection.

    Args:
        A: Input matrix (dense or sparse)

    Returns:
        Dictionary with matrix properties
    """
    if isinstance(A, np.ndarray):
        A_dense = A
        A_sparse = sp.csr_matrix(A)
    else:
        A_sparse = A
        A_dense = A.toarray() if A.shape[0] < 1000 else None

    n = A.shape[0]
    properties = {
        "size": n,
        "density": A_sparse.nnz / (n * n),
        "is_sparse_format": not isinstance(A, np.ndarray)
    }

    if A_dense is not None:
        # Dense matrix analysis
        properties.update({
            "is_symmetric": np.allclose(A_dense, A_dense.T),
            "condition_number": np.linalg.cond(A_dense),
            "determinant": np.linalg.det(A_dense)
        })

        # Check diagonal dominance
        diag = np.abs(np.diag(A_dense))
        off_diag_sum = np.sum(np.abs(A_dense), axis=1) - diag
        properties["diagonal_dominance"] = np.all(diag >= off_diag_sum)
        properties["diagonal_dominance_ratio"] = np.min(diag / (off_diag_sum + 1e-12))

        # Check positive definiteness for symmetric matrices
        if properties["is_symmetric"]:
            try:
                eigenvals = np.linalg.eigvals(A_dense)
                properties["is_positive_definite"] = np.all(eigenvals > 0)
                properties["min_eigenvalue"] = np.min(eigenvals)
                properties["max_eigenvalue"] = np.max(eigenvals)
            except:
                properties["is_positive_definite"] = False
    else:
        # Sparse matrix analysis (approximations)
        properties.update({
            "is_symmetric": "unknown_large_matrix",
            "condition_number": "not_computed_large_matrix",
            "determinant": "not_computed_large_matrix",
            "diagonal_dominance": "not_computed_large_matrix",
            "is_positive_definite": "unknown_large_matrix"
        })

    return properties