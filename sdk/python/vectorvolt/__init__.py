"""VectorVolt Python SDK — HTTP/REST client for VectorVolt."""

from .client import (
    VectorVoltClient,
    Row,
    PageResult,
    HistogramBucket,
    GroupEntry,
    WindowRow,
    ValueCountEntry,
    TableNotFoundError,
    TableAlreadyExistsError,
    VectorVoltError,
)

__all__ = [
    "VectorVoltClient",
    "Row",
    "PageResult",
    "HistogramBucket",
    "GroupEntry",
    "WindowRow",
    "ValueCountEntry",
    "TableNotFoundError",
    "TableAlreadyExistsError",
    "VectorVoltError",
]
__version__ = "0.2.0"
