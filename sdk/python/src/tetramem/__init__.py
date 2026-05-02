"""TetraMem-XL v12.0 Python SDK — 7D Dark Universe Memory System."""

from tetramem.client import TetraMem, AsyncTetraMem
from tetramem.models import (
    MemoryResult,
    RecallResult,
    AssociationResult,
    ConsolidationResult,
    ContextResult,
    StatsResult,
    HealthResult,
)

__version__ = "12.0.0"
__all__ = [
    "TetraMem",
    "AsyncTetraMem",
    "MemoryResult",
    "RecallResult",
    "AssociationResult",
    "ConsolidationResult",
    "ContextResult",
    "StatsResult",
    "HealthResult",
]
