"""
Test Matrix Generator Script
Generates and saves test matrices for benchmark validation.
"""

import numpy as np
import json
from pathlib import Path
from matrix_generators import MatrixGenerators, generate_random_rhs, matrix_properties_summary

def generate_test_matrices(output_dir: str = "test_matrices"):
    """
    Generate comprehensive test matrix suite and save to files.

    Args:
        output_dir: Directory to save test matrices
    """
    output_path = Path(output_dir)
    output_path.mkdir(exist_ok=True)

    print("Generating comprehensive test matrix suite...")

    # Initialize generator
    gen = MatrixGenerators(random_seed=42)

    # Define test sizes
    test_sizes = [50, 100, 200, 500, 1000]

    # Generate test suite
    test_matrices = gen.generate_test_suite(test_sizes, include_sparse=True)

    # Convert matrices to serializable format and save individually
    matrix_catalog = {}

    for size_key, size_matrices in test_matrices.items():
        size_dir = output_path / size_key
        size_dir.mkdir(exist_ok=True)

        matrix_catalog[size_key] = {}

        for matrix_name, matrix_data in size_matrices.items():
            print(f"  Saving {size_key}/{matrix_name}...")

            A = matrix_data["matrix"]

            # Convert sparse matrices to dense for JSON serialization
            if hasattr(A, 'toarray'):
                A_dense = A.toarray()
            else:
                A_dense = A

            # Generate corresponding RHS vectors
            n = A_dense.shape[0]
            rhs_vectors = {
                "random": generate_random_rhs(n, "random"),
                "ones": generate_random_rhs(n, "ones"),
                "smooth": generate_random_rhs(n, "smooth")
            }

            # Create matrix package (convert numpy types to Python types)
            def convert_numpy_types(obj):
                """Convert numpy types to JSON serializable types."""
                if isinstance(obj, (np.integer, np.int64, np.int32)):
                    return int(obj)
                elif isinstance(obj, (np.floating, np.float64, np.float32)):
                    return float(obj)
                elif isinstance(obj, (np.bool_, bool)):
                    return bool(obj)
                elif isinstance(obj, np.ndarray):
                    return obj.tolist()
                elif isinstance(obj, dict):
                    return {k: convert_numpy_types(v) for k, v in obj.items()}
                elif isinstance(obj, list):
                    return [convert_numpy_types(item) for item in obj]
                else:
                    return obj

            matrix_package = {
                "matrix": A_dense.tolist(),
                "size": int(n),
                "description": matrix_data["description"],
                "properties": convert_numpy_types(matrix_data["properties"]),
                "detailed_properties": convert_numpy_types(matrix_properties_summary(A_dense)),
                "rhs_vectors": {name: vec.tolist() for name, vec in rhs_vectors.items()},
                "generation_info": {
                    "generator": "MatrixGenerators",
                    "random_seed": 42,
                    "matrix_type": matrix_name
                }
            }

            # Save matrix package
            matrix_file = size_dir / f"{matrix_name}.json"
            with open(matrix_file, 'w') as f:
                json.dump(matrix_package, f, indent=2)

            # Update catalog
            matrix_catalog[size_key][matrix_name] = {
                "file": str(matrix_file),
                "description": matrix_data["description"],
                "properties": matrix_data["properties"]
            }

    # Save catalog
    catalog_file = output_path / "matrix_catalog.json"
    with open(catalog_file, 'w') as f:
        json.dump(matrix_catalog, f, indent=2)

    print(f"\nTest matrix generation complete!")
    print(f"Matrices saved in: {output_path}")
    print(f"Catalog saved: {catalog_file}")

    # Generate summary statistics
    total_matrices = sum(len(size_matrices) for size_matrices in test_matrices.values())
    print(f"\nGenerated {total_matrices} test matrices across {len(test_sizes)} sizes")

    return str(catalog_file)

def load_test_matrix(catalog_file: str, size: int, matrix_type: str, rhs_type: str = "random"):
    """
    Load a specific test matrix from the generated suite.

    Args:
        catalog_file: Path to matrix catalog JSON
        size: Matrix size
        matrix_type: Type of matrix
        rhs_type: Type of RHS vector ("random", "ones", "smooth")

    Returns:
        Tuple of (A, b) where A is matrix and b is RHS vector
    """
    with open(catalog_file, 'r') as f:
        catalog = json.load(f)

    size_key = f"n_{size}"

    if size_key not in catalog:
        raise ValueError(f"Size {size} not found in catalog")

    if matrix_type not in catalog[size_key]:
        raise ValueError(f"Matrix type {matrix_type} not found for size {size}")

    matrix_file = catalog[size_key][matrix_type]["file"]

    with open(matrix_file, 'r') as f:
        matrix_data = json.load(f)

    A = np.array(matrix_data["matrix"])
    b = np.array(matrix_data["rhs_vectors"][rhs_type])

    return A, b

def validate_generated_matrices(catalog_file: str):
    """
    Validate the generated test matrices.

    Args:
        catalog_file: Path to matrix catalog JSON
    """
    print("Validating generated test matrices...")

    with open(catalog_file, 'r') as f:
        catalog = json.load(f)

    validation_results = {}

    for size_key, size_matrices in catalog.items():
        size = int(size_key.split("_")[1])
        validation_results[size_key] = {}

        for matrix_type, matrix_info in size_matrices.items():
            print(f"  Validating {size_key}/{matrix_type}...")

            try:
                # Load matrix
                A, b = load_test_matrix(catalog_file, size, matrix_type)

                # Basic validations
                validations = {
                    "correct_size": A.shape == (size, size),
                    "square_matrix": A.shape[0] == A.shape[1],
                    "rhs_compatible": b.shape[0] == size,
                    "finite_values": np.all(np.isfinite(A)) and np.all(np.isfinite(b)),
                    "non_zero_diagonal": np.all(np.abs(np.diag(A)) > 1e-12)
                }

                # Check diagonal dominance for DD matrices
                if "dd" in matrix_type or "diagonal" in matrix_type:
                    diag = np.abs(np.diag(A))
                    off_diag_sum = np.sum(np.abs(A), axis=1) - diag
                    validations["diagonal_dominance"] = np.all(diag >= off_diag_sum)

                # Check symmetry for symmetric matrices
                if "symmetric" in matrix_type or "spd" in matrix_type:
                    validations["symmetry"] = np.allclose(A, A.T)

                # Check positive definiteness for SPD matrices
                if "spd" in matrix_type:
                    try:
                        eigenvals = np.linalg.eigvals(A)
                        validations["positive_definite"] = np.all(eigenvals > 0)
                    except:
                        validations["positive_definite"] = False

                validation_results[size_key][matrix_type] = validations

                # Report any failures
                failed_validations = [k for k, v in validations.items() if not v]
                if failed_validations:
                    print(f"    WARNING: Failed validations: {failed_validations}")
                else:
                    print(f"    ✓ All validations passed")

            except Exception as e:
                print(f"    ERROR: Validation failed with exception: {e}")
                validation_results[size_key][matrix_type] = {"error": str(e)}

    # Save validation results
    validation_file = Path(catalog_file).parent / "validation_results.json"
    with open(validation_file, 'w') as f:
        json.dump(validation_results, f, indent=2)

    print(f"\nValidation complete. Results saved to: {validation_file}")

    # Summary
    total_matrices = sum(len(size_matrices) for size_matrices in validation_results.values())
    successful_validations = 0

    for size_matrices in validation_results.values():
        for matrix_validations in size_matrices.values():
            if isinstance(matrix_validations, dict) and "error" not in matrix_validations:
                if all(matrix_validations.values()):
                    successful_validations += 1

    print(f"Validation summary: {successful_validations}/{total_matrices} matrices passed all tests")

    return validation_file

def create_benchmark_examples():
    """Create example matrices for quick testing."""
    print("Creating benchmark examples...")

    gen = MatrixGenerators(random_seed=42)
    examples_dir = Path("examples")
    examples_dir.mkdir(exist_ok=True)

    # Small example for quick testing
    A_small = gen.random_asymmetric_add(10, asymmetry_level=0.5, dominance_factor=2.0)
    b_small = generate_random_rhs(10, "random")

    example_small = {
        "matrix": A_small.tolist(),
        "rhs": b_small.tolist(),
        "size": 10,
        "description": "Small asymmetric diagonally dominant example",
        "expected_properties": {
            "diagonally_dominant": True,
            "asymmetric": True,
            "well_conditioned": True
        }
    }

    with open(examples_dir / "small_example.json", 'w') as f:
        json.dump(example_small, f, indent=2)

    # Medium example with interesting structure
    A_medium = gen.tridiagonal_matrix(50, main_diag=4.0, off_diag=-1.0, perturbation=0.1)
    b_medium = generate_random_rhs(50, "smooth")

    example_medium = {
        "matrix": A_medium.tolist(),
        "rhs": b_medium.tolist(),
        "size": 50,
        "description": "Tridiagonal matrix (PDE-like structure)",
        "expected_properties": {
            "diagonally_dominant": True,
            "symmetric": True,
            "sparse_structure": True
        }
    }

    with open(examples_dir / "medium_example.json", 'w') as f:
        json.dump(example_medium, f, indent=2)

    print(f"Examples saved in: {examples_dir}")

def main():
    """Main function for matrix generation."""
    import argparse

    parser = argparse.ArgumentParser(description="Generate test matrices for linear solver benchmarks")
    parser.add_argument("--output-dir", default="test_matrices", help="Output directory")
    parser.add_argument("--validate", action="store_true", help="Validate generated matrices")
    parser.add_argument("--examples", action="store_true", help="Generate example matrices")

    args = parser.parse_args()

    if args.examples:
        create_benchmark_examples()

    # Generate main test suite
    catalog_file = generate_test_matrices(args.output_dir)

    if args.validate:
        validate_generated_matrices(catalog_file)

    print("\nMatrix generation complete!")
    print(f"Use load_test_matrix() to load specific matrices for benchmarking.")

if __name__ == "__main__":
    main()