"""TetraMem-XL Python SDK — sync and async clients."""

from __future__ import annotations

from typing import Any

import httpx

from .models import (
    AssociationResult,
    ConsolidationResult,
    ContextResult,
    HealthResult,
    MemoryResult,
    RecallResult,
    StatsResult,
)


class _BaseClient:
    def __init__(self, base_url: str = "http://127.0.0.1:3456", timeout: float = 30.0):
        self._base_url = base_url.rstrip("/")
        self._timeout = timeout

    def _remember_payload(
        self,
        content: str,
        *,
        tags: list[str] | None = None,
        category: str = "general",
        importance: float = 0.5,
        source: str = "agent",
    ) -> dict[str, Any]:
        return {
            "content": content,
            "tags": tags or [],
            "category": category,
            "importance": importance,
            "source": source,
        }

    def _recall_payload(
        self,
        query: str,
        *,
        limit: int = 10,
        tags: list[str] | None = None,
        category: str | None = None,
        min_importance: float = 0.0,
    ) -> dict[str, Any]:
        p: dict[str, Any] = {"query": query, "limit": limit, "min_importance": min_importance}
        if tags:
            p["tags"] = tags
        if category:
            p["category"] = category
        return p

    def _associate_payload(
        self,
        topic: str,
        *,
        depth: int = 3,
        limit: int = 10,
    ) -> dict[str, Any]:
        return {"topic": topic, "depth": depth, "limit": limit}

    def _context_add_payload(
        self,
        role: str,
        content: str,
    ) -> dict[str, Any]:
        return {"action": "add", "role": role, "content": content}


class TetraMem(_BaseClient):
    """Synchronous TetraMem client."""

    def __init__(self, base_url: str = "http://127.0.0.1:3456", timeout: float = 30.0):
        super().__init__(base_url, timeout)
        self._client = httpx.Client(base_url=self._base_url, timeout=self._timeout)

    def close(self) -> None:
        self._client.close()

    def __enter__(self) -> TetraMem:
        return self

    def __exit__(self, *args: Any) -> None:
        self.close()

    def stats(self) -> StatsResult:
        r = self._client.get("/api/stats")
        r.raise_for_status()
        return StatsResult(**r.json())

    def health(self) -> HealthResult:
        r = self._client.get("/api/health")
        r.raise_for_status()
        return HealthResult(**r.json())

    def remember(
        self,
        content: str,
        *,
        tags: list[str] | None = None,
        category: str = "general",
        importance: float = 0.5,
        source: str = "agent",
    ) -> MemoryResult:
        r = self._client.post("/api/mcp/tools/call", json={
            "name": "tetramem_remember",
            "arguments": self._remember_payload(
                content, tags=tags, category=category, importance=importance, source=source
            ),
        })
        r.raise_for_status()
        return MemoryResult(**r.json())

    def recall(
        self,
        query: str,
        *,
        limit: int = 10,
        tags: list[str] | None = None,
        category: str | None = None,
        min_importance: float = 0.0,
    ) -> RecallResult:
        r = self._client.post("/api/mcp/tools/call", json={
            "name": "tetramem_recall",
            "arguments": self._recall_payload(
                query, limit=limit, tags=tags, category=category, min_importance=min_importance
            ),
        })
        r.raise_for_status()
        return RecallResult(**r.json())

    def associate(
        self,
        topic: str,
        *,
        depth: int = 3,
        limit: int = 10,
    ) -> AssociationResult:
        r = self._client.post("/api/mcp/tools/call", json={
            "name": "tetramem_associate",
            "arguments": self._associate_payload(topic, depth=depth, limit=limit),
        })
        r.raise_for_status()
        return AssociationResult(**r.json())

    def consolidate(self, *, importance_threshold: float = 0.3) -> ConsolidationResult:
        r = self._client.post("/api/mcp/tools/call", json={
            "name": "tetramem_consolidate",
            "arguments": {"importance_threshold": importance_threshold},
        })
        r.raise_for_status()
        return ConsolidationResult(**r.json())

    def context_add(self, role: str, content: str) -> ContextResult:
        r = self._client.post("/api/mcp/tools/call", json={
            "name": "tetramem_context",
            "arguments": self._context_add_payload(role, content),
        })
        r.raise_for_status()
        return ContextResult(**r.json())

    def context_status(self) -> ContextResult:
        r = self._client.post("/api/mcp/tools/call", json={
            "name": "tetramem_context",
            "arguments": {"action": "status"},
        })
        r.raise_for_status()
        return ContextResult(**r.json())

    def context_reconstruct(self, query: str) -> ContextResult:
        r = self._client.post("/api/mcp/tools/call", json={
            "name": "tetramem_context",
            "arguments": {"action": "reconstruct", "content": query},
        })
        r.raise_for_status()
        return ContextResult(**r.json())

    def context_clear(self) -> ContextResult:
        r = self._client.post("/api/mcp/tools/call", json={
            "name": "tetramem_context",
            "arguments": {"action": "clear"},
        })
        r.raise_for_status()
        return ContextResult(**r.json())


class AsyncTetraMem(_BaseClient):
    """Asynchronous TetraMem client."""

    def __init__(self, base_url: str = "http://127.0.0.1:3456", timeout: float = 30.0):
        super().__init__(base_url, timeout)
        self._client = httpx.AsyncClient(base_url=self._base_url, timeout=self._timeout)

    async def close(self) -> None:
        await self._client.aclose()

    async def __aenter__(self) -> AsyncTetraMem:
        return self

    async def __aexit__(self, *args: Any) -> None:
        await self.close()

    async def _call_tool(self, name: str, arguments: dict[str, Any]) -> dict[str, Any]:
        r = await self._client.post("/api/mcp/tools/call", json={
            "name": name,
            "arguments": arguments,
        })
        r.raise_for_status()
        return r.json()

    async def stats(self) -> StatsResult:
        r = await self._client.get("/api/stats")
        r.raise_for_status()
        return StatsResult(**r.json())

    async def health(self) -> HealthResult:
        r = await self._client.get("/api/health")
        r.raise_for_status()
        return HealthResult(**r.json())

    async def remember(
        self,
        content: str,
        *,
        tags: list[str] | None = None,
        category: str = "general",
        importance: float = 0.5,
        source: str = "agent",
    ) -> MemoryResult:
        data = await self._call_tool(
            "tetramem_remember",
            self._remember_payload(
                content, tags=tags, category=category, importance=importance, source=source
            ),
        )
        return MemoryResult(**data)

    async def recall(
        self,
        query: str,
        *,
        limit: int = 10,
        tags: list[str] | None = None,
        category: str | None = None,
        min_importance: float = 0.0,
    ) -> RecallResult:
        data = await self._call_tool(
            "tetramem_recall",
            self._recall_payload(
                query, limit=limit, tags=tags, category=category, min_importance=min_importance
            ),
        )
        return RecallResult(**data)

    async def associate(
        self,
        topic: str,
        *,
        depth: int = 3,
        limit: int = 10,
    ) -> AssociationResult:
        data = await self._call_tool(
            "tetramem_associate",
            self._associate_payload(topic, depth=depth, limit=limit),
        )
        return AssociationResult(**data)

    async def consolidate(self, *, importance_threshold: float = 0.3) -> ConsolidationResult:
        data = await self._call_tool(
            "tetramem_consolidate",
            {"importance_threshold": importance_threshold},
        )
        return ConsolidationResult(**data)

    async def context_add(self, role: str, content: str) -> ContextResult:
        data = await self._call_tool(
            "tetramem_context",
            self._context_add_payload(role, content),
        )
        return ContextResult(**data)

    async def context_status(self) -> ContextResult:
        data = await self._call_tool("tetramem_context", {"action": "status"})
        return ContextResult(**data)

    async def context_reconstruct(self, query: str) -> ContextResult:
        data = await self._call_tool("tetramem_context", {"action": "reconstruct", "content": query})
        return ContextResult(**data)

    async def context_clear(self) -> ContextResult:
        data = await self._call_tool("tetramem_context", {"action": "clear"})
        return ContextResult(**data)
