# Joicy Architecture

**Creator:** Denis Tu  
**Date:** December 2025

## Overview

Joicy is a team memory bank system that captures, stores, and shares developer knowledge across teams. It enables AI agents to learn from team history, prevent repeated bugs, and provide context-aware assistance. The system works in both SaaS and air-gapped environments, supporting individual developers, teams, and entire organizations.

## Key Concepts

- **Memory Bank**: A vector database that stores code patterns, bug fixes, solutions, and team knowledge
- **Local Memory**: Per-developer memory bank for fast, personal context
- **Central Memory**: Team/company-wide memory bank for shared knowledge
- **MCP Server**: Model Context Protocol server that provides memory bank access to AI agents
- **Sync Service**: Background service that synchronizes local memory with central memory

## System Overview

This diagram shows the high-level architecture of Joicy, including all major components and their relationships. The system is designed with a local-first approach, where each developer has a local memory bank that syncs with a central memory bank for team knowledge sharing.

```mermaid
graph TB
    subgraph "Developer Environment"
        DEV[Developer]
        VSC[VSCode IDE]
        EXT[VSCode Extension]
        CLI[CLI Tool - joicy]
        MCP[MCP Server]
        GIT[Git Hooks]
    end

    subgraph "Local Memory Bank"
        LOCAL_Q[Local Qdrant<br/>Vector DB]
        LOCAL_CACHE[Hot Cache<br/>Redis/Memory]
        LOCAL_INDEX[Metadata Index]
    end

    subgraph "Sync Layer"
        SYNC[Sync Service<br/>Background]
        QUEUE[Message Queue<br/>Async Jobs]
    end

    subgraph "Central Memory Bank"
        CENTRAL_API[Central API<br/>REST/gRPC]
        CENTRAL_Q[Central Qdrant<br/>Vector DB]
        CENTRAL_CACHE[Central Cache]
        AUTH[Auth & Access Control]
    end

    subgraph "Deployment Options"
        SAAS[SaaS Cloud<br/>joicy.dev]
        ONSITE[On-Premise<br/>Docker/K8s]
        AIRGAP[Air-Gapped<br/>Isolated Network]
    end

    DEV --> VSC
    VSC --> EXT
    VSC --> MCP
    EXT --> CLI
    CLI --> MCP
    MCP --> LOCAL_Q
    MCP --> LOCAL_CACHE
    GIT --> CLI
    CLI --> LOCAL_Q
    CLI --> LOCAL_INDEX
    
    LOCAL_Q --> SYNC
    SYNC --> QUEUE
    QUEUE --> CENTRAL_API
    CENTRAL_API --> AUTH
    CENTRAL_API --> CENTRAL_Q
    CENTRAL_API --> CENTRAL_CACHE
    
    CENTRAL_API --> SAAS
    CENTRAL_API --> ONSITE
    CENTRAL_API --> AIRGAP
```

## Component Interaction Flow

This sequence diagram illustrates how different components interact during typical developer workflows. It shows the flow from code writing to memory storage, and from manual queries to result retrieval.

```mermaid
sequenceDiagram
    participant Dev as Developer
    participant VSC as VSCode
    participant Ext as Extension
    participant CLI as CLI Tool
    participant MCP as MCP Server
    participant Local as Local Memory
    participant Sync as Sync Service
    participant Central as Central Memory

    Dev->>VSC: Writes code
    VSC->>Ext: Code change event
    Ext->>MCP: Query memory bank
    MCP->>Local: Search patterns
    Local-->>MCP: Return results
    MCP-->>Ext: Suggest fixes/patterns
    Ext-->>VSC: Show suggestions
    
    Dev->>VSC: Commits code
    VSC->>CLI: Git hook triggered
    CLI->>Local: Store code context
    CLI->>Local: Extract patterns
    Local-->>CLI: Stored
    CLI->>Sync: Queue sync job
    Sync->>Central: Sync to central
    Central-->>Sync: Confirmed
    
    Dev->>CLI: joicy search "bug pattern"
    CLI->>Local: Search local
    Local-->>CLI: Local results
    CLI->>Central: Search central
    Central-->>CLI: Central results
    CLI-->>Dev: Combined results
```

## Memory Bank Hierarchy

Joicy uses a hierarchical memory structure that scales from individual developers to entire organizations. Each level syncs knowledge upward while allowing queries across all levels. This enables both personal context (fast, local) and team knowledge (comprehensive, shared).

```mermaid
graph TD
    subgraph "Individual Level"
        DEV1[Dev 1<br/>Local Memory]
        DEV2[Dev 2<br/>Local Memory]
        DEV3[Dev 3<br/>Local Memory]
    end

    subgraph "Team Level"
        TEAM1[Backend Team<br/>Memory Bank]
        TEAM2[Frontend Team<br/>Memory Bank]
        TEAM3[DevOps Team<br/>Memory Bank]
    end

    subgraph "Company Level"
        COMPANY[Company Memory Bank<br/>Central Repository]
    end

    DEV1 -->|Sync| TEAM1
    DEV2 -->|Sync| TEAM1
    DEV3 -->|Sync| TEAM2
    
    TEAM1 -->|Sync| COMPANY
    TEAM2 -->|Sync| COMPANY
    TEAM3 -->|Sync| COMPANY
    
    DEV1 -.->|Query| TEAM1
    DEV2 -.->|Query| COMPANY
```

## Search Performance Architecture

To handle large memory banks efficiently, Joicy uses a three-tier caching strategy:
- **Hot Cache**: In-memory cache for recent, frequently accessed patterns (<10ms)
- **Warm Index**: Fast vector search on recent data with metadata filtering (50-200ms)
- **Cold Storage**: Full archive search for historical patterns (500ms-2s, async)

This ensures commit-time searches remain fast (<300ms) even as the memory bank grows to millions of entries.

```mermaid
graph LR
    REQ[Search Request]
    
    subgraph "Tier 1: Hot Cache"
        HOT[Hot Cache<br/><10ms<br/>Recent 30 days]
    end
    
    subgraph "Tier 2: Warm Index"
        WARM[Warm Index<br/>50-200ms<br/>Last 6 months]
        META[Metadata Filter<br/>File/Feature/Language]
    end
    
    subgraph "Tier 3: Cold Storage"
        COLD[Cold Storage<br/>500ms-2s<br/>Archive]
    end
    
    REQ --> HOT
    HOT -->|Cache Miss| META
    META --> WARM
    WARM -->|Not Found| COLD
    
    HOT -->|Cache Hit| RESULT[Results<br/><300ms]
    WARM -->|Found| RESULT
    COLD -->|Async| NOTIFY[Background<br/>Notification]
```

## Deployment Architecture

Joicy supports three deployment models to meet different security and compliance requirements:
- **SaaS**: Cloud-hosted service for teams that want managed infrastructure
- **On-Premise**: Self-hosted deployment for companies with data sovereignty requirements
- **Air-Gapped**: Fully isolated deployment for highly regulated environments

All deployment models use the same client components (CLI, MCP server, VSCode extension), only the backend location changes.

```mermaid
graph TB
    subgraph "SaaS Deployment"
        SAAS_CLOUD[Cloud Infrastructure]
        SAAS_API[Central API<br/>joicy.dev]
        SAAS_DB[Managed Vector DB]
        SAAS_AUTH[OAuth/SAML]
    end

    subgraph "On-Premise Deployment"
        ONSITE_K8S[Kubernetes Cluster]
        ONSITE_API[API Service]
        ONSITE_DB[Self-Hosted Qdrant]
        ONSITE_AUTH[LDAP/AD]
    end

    subgraph "Air-Gapped Deployment"
        AIRGAP_DOCKER[Docker Container]
        AIRGAP_LOCAL[Local Vector DB]
        AIRGAP_SYNC[Manual Sync<br/>USB/Network]
    end

    subgraph "Developer Client"
        CLIENT_CLI[CLI Tool]
        CLIENT_MCP[MCP Server]
        CLIENT_EXT[VSCode Extension]
    end

    CLIENT_CLI --> SAAS_API
    CLIENT_CLI --> ONSITE_API
    CLIENT_CLI --> AIRGAP_LOCAL
    
    CLIENT_MCP --> CLIENT_CLI
    CLIENT_EXT --> CLIENT_CLI
    
    SAAS_API --> SAAS_DB
    ONSITE_API --> ONSITE_DB
    AIRGAP_LOCAL --> AIRGAP_DOCKER
```

## Data Flow: Commit to Memory Bank

This flowchart shows the complete process when a developer commits code. The system extracts context, analyzes patterns, searches for similar issues, stores new knowledge, and syncs with the central memory bank. This is one of many trigger mechanisms - the memory bank is also queried in real-time during coding, via AI agents, and through manual searches.

```mermaid
flowchart TD
    START[Developer Commits Code]
    
    GIT_HOOK[Git Pre-Commit Hook]
    EXTRACT[Extract Code Context<br/>- File paths<br/>- Code diff<br/>- Commit message]
    
    ANALYZE[Analyze Patterns<br/>- Bug patterns<br/>- Code smells<br/>- Solutions]
    
    FILTER[Metadata Filtering<br/>- Project<br/>- Feature<br/>- Language]
    
    SEARCH[Search Local Memory<br/>- Similar patterns<br/>- Known bugs<br/>- Previous fixes]
    
    STORE[Store in Local Memory<br/>- Embed code<br/>- Index metadata<br/>- Cache hot patterns]
    
    SYNC[Queue Sync Job<br/>Background Process]
    
    CENTRAL[Sync to Central<br/>- Team memory<br/>- Company memory]
    
    ALERT{Pattern<br/>Found?}
    
    SUGGEST[Suggest Fix<br/>Show in VSCode]
    
    START --> GIT_HOOK
    GIT_HOOK --> EXTRACT
    EXTRACT --> ANALYZE
    ANALYZE --> FILTER
    FILTER --> SEARCH
    SEARCH --> ALERT
    ALERT -->|Yes| SUGGEST
    ALERT -->|No| STORE
    STORE --> SYNC
    SYNC --> CENTRAL
```

## Product Components

Joicy is delivered as multiple components that work together:
- **CLI Tool**: Primary interface for developers (Rust binary)
- **MCP Server**: Embedded server that provides memory bank access to AI agents
- **VSCode Extension**: Optional UI for enhanced developer experience
- **Docker Image**: For enterprise self-hosted deployments

All components can be installed via package managers, marketplaces, or manual download.

```mermaid
graph LR
    subgraph "Deliverables"
        BINARY[joicy CLI<br/>Rust Binary]
        MCP_BIN[MCP Server<br/>Embedded]
        VSIX[VSCode Extension<br/>.vsix file]
        DOCKER[Docker Image<br/>joicy/joicy]
    end

    subgraph "Installation Methods"
        PACKAGE[Package Managers<br/>brew, cargo, npm]
        MARKETPLACE[VSCode Marketplace]
        DOCKER_HUB[Docker Hub]
        MANUAL[Manual Download]
    end

    subgraph "User Interfaces"
        CLI_UI[Command Line<br/>joicy init, sync, search]
        VSCODE_UI[VSCode UI<br/>Sidebar, suggestions]
        MCP_UI[MCP Protocol<br/>AI agent integration]
    end

    BINARY --> PACKAGE
    BINARY --> MANUAL
    MCP_BIN --> BINARY
    VSIX --> MARKETPLACE
    DOCKER --> DOCKER_HUB
    
    PACKAGE --> CLI_UI
    MARKETPLACE --> VSCODE_UI
    BINARY --> MCP_UI
```

## Scaling Strategy

Joicy scales from small teams to large enterprises:
- **Small Teams**: Single instance handles both local and central memory
- **Medium Teams**: Distributed architecture with team-level memory banks
- **Large Enterprises**: Multi-tier architecture with sharding, clustering, and CDN caching

The architecture grows with the organization without requiring major changes.

```mermaid
graph TB
    subgraph "Small Team < 10 devs"
        SMALL[Single Instance<br/>Local + Central<br/>Same server]
    end

    subgraph "Medium Team 10-100 devs"
        MEDIUM[Distributed<br/>- Local per dev<br/>- Central per team<br/>- Load balancer]
    end

    subgraph "Large Enterprise 100+ devs"
        LARGE[Multi-Tier<br/>- Local per dev<br/>- Team clusters<br/>- Company shards<br/>- CDN cache]
    end

    SMALL --> MEDIUM
    MEDIUM --> LARGE
```

## Use Cases & Trigger Mechanisms

Joicy is not limited to git hooks - it provides value across multiple use cases:

**Real-Time Use Cases**: Memory bank is queried continuously as developers code, providing instant suggestions and context-aware autocomplete.

**Git-Based Use Cases**: Pre/post-commit hooks analyze code and store context, while PR integration provides review suggestions.

**AI Agent Use Cases**: MCP server enables AI agents to query memory bank during conversations, load team context at session start, and retrieve relevant patterns.

**Manual Use Cases**: Developers can search memory bank via CLI, ask questions through chat interfaces, and generate documentation.

**Onboarding Use Cases**: New developers can query team knowledge, discover similar features, and learn best practices.

**Code Review Use Cases**: PR analysis compares with previous PRs, detects known patterns, and suggests solutions from memory bank.

```mermaid
graph TB
    subgraph "Real-Time Use Cases"
        RT1[While Coding<br/>AI agent queries memory<br/>as developer types]
        RT2[IDE Suggestions<br/>Inline code suggestions<br/>based on team patterns]
        RT3[Autocomplete<br/>Context-aware completions<br/>from memory bank]
    end

    subgraph "Git-Based Use Cases"
        GIT1[Pre-Commit Hook<br/>Detect known bugs<br/>before commit]
        GIT2[Post-Commit Hook<br/>Store code context<br/>after commit]
        GIT3[PR/MR Integration<br/>Review suggestions<br/>based on history]
    end

    subgraph "AI Agent Use Cases"
        AI1[MCP Server Queries<br/>AI agent asks for context<br/>during conversation]
        AI2[Memory Initialization<br/>Load team knowledge<br/>at session start]
        AI3[Context Retrieval<br/>Fetch relevant patterns<br/>for current task]
    end

    subgraph "Manual Use Cases"
        MAN1[CLI Search<br/>Developer searches<br/>via joicy search command]
        MAN2[Chat Interface<br/>Ask questions about<br/>codebase patterns]
        MAN3[Documentation<br/>Generate docs from<br/>memory bank knowledge]
    end

    subgraph "Onboarding Use Cases"
        ONB1[New Developer<br/>Query team knowledge<br/>about project]
        ONB2[Feature Discovery<br/>Find similar features<br/>and implementations]
        ONB3[Best Practices<br/>Learn team patterns<br/>and conventions]
    end

    subgraph "Code Review Use Cases"
        CR1[PR Analysis<br/>Compare with similar<br/>previous PRs]
        CR2[Pattern Detection<br/>Identify known issues<br/>in review]
        CR3[Solution Suggestions<br/>Recommend fixes<br/>from memory bank]
    end

    RT1 --> MEM[Memory Bank]
    RT2 --> MEM
    RT3 --> MEM
    GIT1 --> MEM
    GIT2 --> MEM
    GIT3 --> MEM
    AI1 --> MEM
    AI2 --> MEM
    AI3 --> MEM
    MAN1 --> MEM
    MAN2 --> MEM
    MAN3 --> MEM
    ONB1 --> MEM
    ONB2 --> MEM
    ONB3 --> MEM
    CR1 --> MEM
    CR2 --> MEM
    CR3 --> MEM
```

## Complete Interaction Model

This diagram shows all the ways developers interact with Joicy and how those interactions trigger memory bank operations. The system supports multiple trigger mechanisms (file watching, git hooks, CLI commands, MCP requests, PR webhooks) that all feed into the same memory bank operations (query, store, sync, analyze).

```mermaid
graph LR
    subgraph "Developer Actions"
        CODE[Writing Code]
        COMMIT[Committing]
        SEARCH[Searching]
        CHAT[Chatting with AI]
        REVIEW[Code Review]
    end

    subgraph "Trigger Mechanisms"
        FILE_WATCH[File Watcher<br/>Real-time]
        GIT_HOOK[Git Hooks<br/>Commit events]
        CLI_CMD[CLI Commands<br/>Manual]
        MCP_REQ[MCP Requests<br/>AI agent]
        PR_WEBHOOK[PR Webhooks<br/>CI/CD]
    end

    subgraph "Memory Bank Operations"
        QUERY[Query Memory<br/>Search patterns]
        STORE[Store Context<br/>Save knowledge]
        SYNC[Sync Data<br/>Share with team]
        ANALYZE[Analyze Code<br/>Extract patterns]
    end

    CODE --> FILE_WATCH
    COMMIT --> GIT_HOOK
    SEARCH --> CLI_CMD
    CHAT --> MCP_REQ
    REVIEW --> PR_WEBHOOK

    FILE_WATCH --> QUERY
    GIT_HOOK --> ANALYZE
    GIT_HOOK --> STORE
    CLI_CMD --> QUERY
    MCP_REQ --> QUERY
    PR_WEBHOOK --> QUERY

    ANALYZE --> STORE
    STORE --> SYNC
    QUERY --> CODE
    QUERY --> CHAT
    QUERY --> REVIEW
```

