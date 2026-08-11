"""VectorVolt Python SDK — HTTP/REST client for VectorVolt."""

from .client import VectorVoltClient, Row, TableNotFoundError, TableAlreadyExistsError, VectorVoltError

__all__ = ["VectorVoltClient", "Row", "TableNotFoundError", "TableAlreadyExistsError", "VectorVoltError"]
__version__ = "0.1.0"
