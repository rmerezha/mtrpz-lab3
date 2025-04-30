from fastapi import APIRouter
import numpy as np
from typing import List

router = APIRouter()


@router.get('')
def hello_world() -> dict:
    return {'msg': 'Hello, World!'}

def generate_matrix(rows: int, cols: int) -> np.ndarray:
    return np.random.randint(0, 10, size=(rows, cols))

def multiply_matrices(a: np.ndarray, b: np.ndarray) -> np.ndarray:
    return np.matmul(a, b)

def matrix_to_list(matrix: np.ndarray) -> List[List[int]]:
    return matrix.tolist()

@router.get("/matrix")
def get_matrices() -> dict:
    matrix_a = generate_matrix(10, 10)
    matrix_b = generate_matrix(10, 10)
    product = multiply_matrices(matrix_a, matrix_b)

    return {
        "matrix_a": matrix_to_list(matrix_a),
        "matrix_b": matrix_to_list(matrix_b),
        "product": matrix_to_list(product)
    }