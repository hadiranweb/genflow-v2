## GenFlow v2 — MCP Resolution Flow

### What is MCP?

Master Context Protocol (MCP) is GenFlow's context management system. Each MCP Type is an independent **Cell** with its own lifecycle, scope, and cache TTL.

### Resolution Pipeline

```
┌──────────┐     ┌──────────┐     ┌──────────┐
│  Cache   │────→│ Database │────→│  Build   │
│ (Redis)  │     │ (PgPool) │     │ (Draft)  │
└──────────┘     └──────────┘     └──────────┘
     ↓ HIT            ↓ HIT            ↓
  Return MCP      Return MCP      Draft MCP
  (fastest)       (medium)        (slowest, temporary)
```

### MCP Types

| Cell | Scope | Cache TTL | Reusable | Use Case |
|------|-------|-----------|----------|----------|
| `PlatformPolicy` | Global | 7 days | ✅ | Legal, privacy, fairness policies |
| `Industry` | Global/Industry | 24h | ✅ | Industry standards (retail, SaaS, healthcare) |
| `BusinessProcess` | Global/Industry | 24h | ✅ | Process templates (inventory, sales pipeline) |
| `StandardPosition` | Global/Industry | 24h | ✅ | Standard position templates |
| `OrganizationContext` | Tenant | 1h | ✅ | Tenant-specific data and context |
| `CaseTemporary` | Case | 30m | ❌ | Temporary context for a specific analysis session |

### Bundle Composition

A full MCP bundle for a business analysis includes:

1. **Industry MCP**: Industry standards and constraints
2. **Process MCPs**: Business process templates (multiple)
3. **Standard Position MCPs**: Position templates (multiple)
4. **Organization Context MCP**: Tenant-specific context
5. **Case MCP**: Temporary session context
6. **Policy Guardrails**: Legal and fairness constraints

### Cache Key Format

```
mcp:ctx:{mcp_type}:{scope}:{code}:active
```

Example: `mcp:ctx:industry:industry:retail:active`

### Lifecycle

```
Draft → ReviewReady → Approved → Active → Deprecated → Archived
```

Only `Active` and `Approved` MCPs are used in production resolution.
