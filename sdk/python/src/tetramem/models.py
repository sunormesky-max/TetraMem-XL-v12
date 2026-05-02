"""Data models for TetraMem SDK."""

from __future__ import annotations

from typing import Any

from pydantic import BaseModel, Field


class StatsResult(BaseModel):
    active_nodes: int = 0
    manifested_nodes: int = 0
    dark_nodes: int = 0
    total_energy: float = 0.0
    allocated_energy: float = 0.0
    available_energy: float = 0.0
    physical_energy: float = 0.0
    dark_energy: float = 0.0
    utilization: float = 0.0
    energy_drift: float = 0.0
    memory_count: int = 0
    hebbian_edges: int = 0
    conservation_ok: bool = True


class HealthResult(BaseModel):
    health_level: str = "Unknown"
    node_count: int = 0
    energy_utilization: float = 0.0
    conservation_ok: bool = True
    hebbian_edge_count: int = 0
    memory_count: int = 0


class MemoryResult(BaseModel):
    success: bool = False
    memory_id: str = ""
    anchor: str = ""
    semantic_links: int = 0
    conservation_ok: bool = True


class RecallHit(BaseModel):
    anchor: str = ""
    similarity: float = 0.0
    dimensions: int = 0
    hebbian_neighbors: int = 0
    associated_memories: list[str] = Field(default_factory=list)


class RecallResult(BaseModel):
    query: str = ""
    results: list[RecallHit] = Field(default_factory=list)
    total_found: int = 0
    returned: int = 0


class AssociationResult(BaseModel):
    topic: str = ""
    seed_anchor: str = ""
    associations: list[dict[str, Any]] = Field(default_factory=list)
    pulse_spread: dict[str, Any] = Field(default_factory=dict)
    total: int = 0


class ConsolidationResult(BaseModel):
    consolidation: str = ""
    edges_before: int = 0
    edges_after: int = 0
    strengthened_paths: int = 0
    weakened_paths: int = 0
    conservation_ok: bool = True


class ContextResult(BaseModel):
    action: str = ""
    context_entries: int = 0
    current_tokens: int = 0
    max_tokens: int = 0
    overflow_archived: int = 0
