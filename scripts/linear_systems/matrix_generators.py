"""
Matrix Generators for Linear System Testing
Generates various types of matrices for comprehensive benchmarking.
"""

import numpy as np
import scipy.sparse as sp
from typing import Dict, Any, Optional, Tuple
import warnings

class MatrixGenerators:
    """Collection of matrix generators for testing different solver characteristics."""

    def __init__(self, random_seed: Optional[int] = None):
        """
        Initialize matrix generator.

        Args:
            random_seed: Random seed for reproducible results
        """
        if random_seed is not None:
            np.random.seed(random_seed)

    def diagonally_dominant_dense(self, n: int, dominance_factor: float = 2.0,
                                 symmetric: bool = False, condition_number: float = None) -> np.ndarray:
        """
        Generate dense diagonally dominant matrix.

        Args:
            n: Matrix size
            dominance_factor: How much larger diagonal elements are than off-diagonal sum
            symmetric: Whether to make matrix symmetric
            condition_number: Target condition number (optional)

        Returns:
            Diagonally dominant matrix
        """
        # Generate random off-diagonal elements
        if symmetric:
            A = np.random.randn(n, n)
            A = (A + A.T) / 2  # Make symmetric
            np.fill_diagonal(A, 0)  # Clear diagonal
        else:
            A = np.random.randn(n, n)
            np.fill_diagonal(A, 0)

        # Set diagonal to ensure dominance
        off_diag_sums = np.sum(np.abs(A), axis=1)
        diag_values = dominance_factor * off_diag_sums + np.abs(np.random.randn(n))

        # Ensure positive diagonal for stability
        diag_values = np.abs(diag_values) + 0.1

        np.fill_diagonal(A, diag_values)

        # Adjust condition number if specified
        if condition_number is not None and symmetric:
            A = self._adjust_condition_number(A, condition_number)

        return A

    def tridiagonal_matrix(self, n: int, main_diag: float = 4.0,
                          off_diag: float = -1.0, perturbation: float = 0.1) -> np.ndarray:
        """
        Generate tridiagonal matrix (common in discretized PDEs).

        Args:
            n: Matrix size
            main_diag: Main diagonal value
            off_diag: Off-diagonal value
            perturbation: Random perturbation level

        Returns:
            Tridiagonal matrix
        """
        A = np.zeros((n, n))

        # Main diagonal
        main_values = main_diag + perturbation * np.random.randn(n)
        np.fill_diagonal(A, main_values)

        # Upper and lower diagonals
        if n > 1:
            upper_values = off_diag + perturbation * np.random.randn(n-1)
            lower_values = off_diag + perturbation * np.random.randn(n-1)

            A[np.arange(n-1), np.arange(1, n)] = upper_values
            A[np.arange(1, n), np.arange(n-1)] = lower_values

        return A

    def sparse_diagonally_dominant(self, n: int, density: float = 0.1,
                                  dominance_factor: float = 2.0) -> sp.csr_matrix:
        """
        Generate sparse diagonally dominant matrix.

        Args:
            n: Matrix size
            density: Sparsity density (fraction of non-zero elements)
            dominance_factor: Diagonal dominance factor

        Returns:
            Sparse diagonally dominant matrix
        """
        # Generate random sparse structure
        nnz = int(n * n * density)
        rows = np.random.randint(0, n, nnz)
        cols = np.random.randint(0, n, nnz)
        data = np.random.randn(nnz)

        # Remove diagonal entries for now
        mask = rows != cols
        rows, cols, data = rows[mask], cols[mask], data[mask]

        # Create sparse matrix
        A = sp.coo_matrix((data, (rows, cols)), shape=(n, n))
        A = A.tocsr()

        # Add diagonal dominance
        off_diag_sums = np.array(A.sum(axis=1)).flatten()
        off_diag_sums = np.abs(off_diag_sums)

        diag_values = dominance_factor * off_diag_sums + np.abs(np.random.randn(n)) + 0.1
        A.setdiag(diag_values)

        return A

    def banded_matrix(self, n: int, bandwidth: int = 5, dominance_factor: float = 1.5) -> np.ndarray:
        """
        Generate banded diagonally dominant matrix.

        Args:
            n: Matrix size
            bandwidth: Number of diagonals on each side of main diagonal
            dominance_factor: Diagonal dominance factor

        Returns:
            Banded matrix
        """
        A = np.zeros((n, n))

        # Fill bands
        for k in range(-bandwidth, bandwidth + 1):
            if k == 0:
                continue  # Skip main diagonal for now

            diag_length = n - abs(k)
            if diag_length > 0:
                values = np.random.randn(diag_length)
                if k > 0:
                    A[np.arange(diag_length), np.arange(k, n)] = values
                else:
                    A[np.arange(-k, n), np.arange(diag_length)] = values

        # Set main diagonal for dominance
        off_diag_sums = np.sum(np.abs(A), axis=1)
        diag_values = dominance_factor * off_diag_sums + np.abs(np.random.randn(n)) + 0.1
        np.fill_diagonal(A, diag_values)

        return A

    def symmetric_positive_definite(self, n: int, condition_number: float = 10.0) -> np.ndarray:
        """
        Generate symmetric positive definite matrix.

        Args:
            n: Matrix size
            condition_number: Target condition number

        Returns:
            Symmetric positive definite matrix
        """
        # Generate random orthogonal matrix
        Q, _ = np.linalg.qr(np.random.randn(n, n))

        # Generate eigenvalues with specified condition number
        lambda_max = condition_number
        lambda_min = 1.0
        eigenvals = np.logspace(np.log10(lambda_min), np.log10(lambda_max), n)

        # Construct SPD matrix
        A = Q @ np.diag(eigenvals) @ Q.T

        return A

    def ill_conditioned_matrix(self, n: int, condition_number: float = 1e12) -> np.ndarray:
        """
        Generate ill-conditioned matrix for stress testing.

        Args:
            n: Matrix size
            condition_number: Target condition number (large for ill-conditioning)

        Returns:
            Ill-conditioned matrix
        """
        # Start with a well-conditioned matrix
        A = self.diagonally_dominant_dense(n, dominance_factor=1.1, symmetric=True)

        # Adjust condition number
        A = self._adjust_condition_number(A, condition_number)

        return A

    def random_asymmetric_add(self, n: int, asymmetry_level: float = 0.5,
                             dominance_factor: float = 2.0) -> np.ndarray:
        """
        Generate random asymmetric diagonally dominant matrix.

        Args:
            n: Matrix size
            asymmetry_level: Level of asymmetry (0 = symmetric, 1 = fully asymmetric)
            dominance_factor: Diagonal dominance factor

        Returns:
            Asymmetric diagonally dominant matrix
        """
        # Generate symmetric part
        A_sym = np.random.randn(n, n)
        A_sym = (A_sym + A_sym.T) / 2

        # Generate asymmetric part
        A_asym = np.random.randn(n, n)
        A_asym = (A_asym - A_asym.T) / 2

        # Combine
        A = A_sym + asymmetry_level * A_asym
        np.fill_diagonal(A, 0)

        # Ensure diagonal dominance
        off_diag_sums = np.sum(np.abs(A), axis=1)
        diag_values = dominance_factor * off_diag_sums + np.abs(np.random.randn(n)) + 0.1
        np.fill_diagonal(A, diag_values)

        return A

    def discretized_laplacian(self, n: int, dimension: int = 1) -> np.ndarray:
        """
        Generate discretized Laplacian matrix (from PDE discretization).

        Args:
            n: Matrix size
            dimension: Spatial dimension (1D, 2D, or 3D)

        Returns:
            Discretized Laplacian matrix
        """
        if dimension == 1:
            # 1D Laplacian: tridiagonal [-1, 2, -1]
            A = np.zeros((n, n))
            np.fill_diagonal(A, 2.0)
            if n > 1:
                A[np.arange(n-1), np.arange(1, n)] = -1.0
                A[np.arange(1, n), np.arange(n-1)] = -1.0

        elif dimension == 2:
            # 2D Laplacian on grid
            side = int(np.sqrt(n))
            if side * side != n:
                warnings.warn(f"Matrix size {n} is not a perfect square, using {side}²={side*side}")
                n = side * side

            A = np.zeros((n, n))

            # 5-point stencil
            for i in range(side):
                for j in range(side):
                    idx = i * side + j
                    A[idx, idx] = 4.0

                    # Neighbors
                    if i > 0:  # Top
                        A[idx, (i-1) * side + j] = -1.0
                    if i < side - 1:  # Bottom
                        A[idx, (i+1) * side + j] = -1.0
                    if j > 0:  # Left
                        A[idx, i * side + (j-1)] = -1.0
                    if j < side - 1:  # Right
                        A[idx, i * side + (j+1)] = -1.0

        else:
            raise ValueError(f"Dimension {dimension} not supported")

        return A

    def generate_test_suite(self, sizes: list, include_sparse: bool = True) -> Dict[str, Dict[str, Any]]:
        """
        Generate complete test suite of matrices.

        Args:
            sizes: List of matrix sizes to generate
            include_sparse: Whether to include sparse matrices

        Returns:
            Dictionary with matrix categories and instances
        """
        test_matrices = {}

        for size in sizes:
            size_key = f"n_{size}"
            test_matrices[size_key] = {}

            # Dense diagonally dominant (symmetric and asymmetric)
            test_matrices[size_key]["dd_symmetric"] = {
                "matrix": self.diagonally_dominant_dense(size, dominance_factor=2.0, symmetric=True),
                "description": "Dense symmetric diagonally dominant",
                "properties": {"symmetric": True, "diagonally_dominant": True, "sparse": False}
            }

            test_matrices[size_key]["dd_asymmetric"] = {
                "matrix": self.random_asymmetric_add(size, asymmetry_level=0.7),
                "description": "Dense asymmetric diagonally dominant",
                "properties": {"symmetric": False, "diagonally_dominant": True, "sparse": False}
            }

            # Tridiagonal
            test_matrices[size_key]["tridiagonal"] = {
                "matrix": self.tridiagonal_matrix(size),
                "description": "Tridiagonal matrix",
                "properties": {"symmetric": True, "diagonally_dominant": True, "sparse": True, "banded": True}
            }

            # Symmetric positive definite
            test_matrices[size_key]["spd_well_conditioned"] = {
                "matrix": self.symmetric_positive_definite(size, condition_number=10.0),
                "description": "Well-conditioned SPD matrix",
                "properties": {"symmetric": True, "positive_definite": True, "sparse": False}
            }

            test_matrices[size_key]["spd_ill_conditioned"] = {
                "matrix": self.symmetric_positive_definite(size, condition_number=1e6),
                "description": "Ill-conditioned SPD matrix",
                "properties": {"symmetric": True, "positive_definite": True, "sparse": False, "ill_conditioned": True}
            }

            # Banded matrix
            bandwidth = max(3, size // 10)
            test_matrices[size_key]["banded"] = {
                "matrix": self.banded_matrix(size, bandwidth=bandwidth),
                "description": f"Banded matrix (bandwidth={bandwidth})",
                "properties": {"symmetric": False, "diagonally_dominant": True, "sparse": True, "banded": True}
            }

            # Sparse matrices (for larger sizes)
            if include_sparse and size >= 100:
                test_matrices[size_key]["sparse_dd"] = {
                    "matrix": self.sparse_diagonally_dominant(size, density=0.05),
                    "description": "Sparse diagonally dominant",
                    "properties": {"symmetric": False, "diagonally_dominant": True, "sparse": True}
                }

            # Discretized Laplacian (for relevant sizes)
            if size <= 10000:  # Avoid huge matrices
                test_matrices[size_key]["laplacian_1d"] = {
                    "matrix": self.discretized_laplacian(size, dimension=1),
                    "description": "1D discretized Laplacian",
                    "properties": {"symmetric": True, "diagonally_dominant": True, "sparse": True, "pde_origin": True}
                }

                # 2D Laplacian for perfect squares
                sqrt_size = int(np.sqrt(size))
                if sqrt_size * sqrt_size == size and size >= 25:
                    test_matrices[size_key]["laplacian_2d"] = {
                        "matrix": self.discretized_laplacian(size, dimension=2),
                        "description": "2D discretized Laplacian",
                        "properties": {"symmetric": True, "diagonally_dominant": True, "sparse": True, "pde_origin": True}
                    }

        return test_matrices

    def _adjust_condition_number(self, A: np.ndarray, target_cond: float) -> np.ndarray:
        """
        Adjust matrix condition number via eigenvalue modification.

        Args:
            A: Input matrix (should be symmetric)
            target_cond: Target condition number

        Returns:
            Matrix with adjusted condition number
        """
        try:
            # Eigendecomposition
            eigenvals, eigenvecs = np.linalg.eigh(A)

            # Adjust eigenvalues to achieve target condition number
            lambda_min = 1.0
            lambda_max = target_cond
            eigenvals_new = np.linspace(lambda_min, lambda_max, len(eigenvals))

            # Reconstruct matrix
            A_new = eigenvecs @ np.diag(eigenvals_new) @ eigenvecs.T

            return A_new

        except:
            # Fallback: return original matrix if adjustment fails
            warnings.warn("Condition number adjustment failed, returning original matrix")
            return A

def generate_random_rhs(n: int, structure: str = "random") -> np.ndarray:
    """
    Generate right-hand side vector for testing.

    Args:
        n: Vector size
        structure: "random", "ones", "alternating", or "smooth"

    Returns:
        Right-hand side vector
    """
    if structure == "random":
        return np.random.randn(n)
    elif structure == "ones":
        return np.ones(n)
    elif structure == "alternating":
        return np.array([(-1)**i for i in range(n)])
    elif structure == "smooth":
        # Smooth function (e.g., sine wave)
        x = np.linspace(0, 2*np.pi, n)
        return np.sin(x) + 0.5 * np.sin(3*x)
    else:
        raise ValueError(f"Unknown RHS structure: {structure}")

def matrix_properties_summary(A) -> Dict[str, Any]:
    """
    Compute summary of matrix properties.

    Args:
        A: Input matrix

    Returns:
        Dictionary with matrix properties
    """
    if hasattr(A, 'toarray'):
        # Sparse matrix
        A_dense = A.toarray() if A.shape[0] < 1000 else None
        is_sparse_format = True
        density = A.nnz / (A.shape[0] * A.shape[1])
    else:
        # Dense matrix
        A_dense = A
        is_sparse_format = False
        density = 1.0

    n = A.shape[0]
    properties = {
        "size": n,
        "is_square": A.shape[0] == A.shape[1],
        "is_sparse_format": is_sparse_format,
        "density": density
    }

    if A_dense is not None and n < 1000:
        # Detailed analysis for smaller matrices
        properties.update({
            "norm_frobenius": np.linalg.norm(A_dense, 'fro'),
            "norm_1": np.linalg.norm(A_dense, 1),
            "norm_inf": np.linalg.norm(A_dense, np.inf),
            "trace": np.trace(A_dense),
            "rank": np.linalg.matrix_rank(A_dense)
        })

        # Symmetry
        if n < 500:  # Only for smaller matrices
            properties["is_symmetric"] = np.allclose(A_dense, A_dense.T)

        # Diagonal dominance
        diag = np.abs(np.diag(A_dense))
        off_diag_sum = np.sum(np.abs(A_dense), axis=1) - diag
        properties["diagonal_dominance"] = {
            "strict": np.all(diag > off_diag_sum),
            "weak": np.all(diag >= off_diag_sum),
            "min_ratio": np.min(diag / (off_diag_sum + 1e-12))
        }

        # Condition number (expensive for large matrices)
        if n < 200:
            try:
                properties["condition_number"] = np.linalg.cond(A_dense)
            except:
                properties["condition_number"] = "computation_failed"

    return properties