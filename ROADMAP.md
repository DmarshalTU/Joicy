# Joicy Roadmap

**Creator:** Denis Tu  
**Date:** December 2025

## Overview

This roadmap outlines the development plan for Joicy, from initial MVP to full enterprise features. The project is organized into phases, with each phase building upon the previous one.

## Status (April 2026): local developer POC

The **single-machine local POC** described in recent work is **implemented**: SQLite + FTS5 memory, Obsidian vault export, four-step post-commit automation (capture → vault → changelog → ticket stub), git hooks using the `git` CLI, subprocess **system tests**, and MCP tools (`memory_search`, `memory_store`, `memory_changelog`, `memory_vault_note` with optional stable `file_stem`). **Not in this POC**: central `joicy sync`, remote team sync, embedding/semantic search on the default path, and production Qdrant/Chroma wiring (stubs / feature flags only).

Phase checklists below are the original plan; treat Phase 0 items as partially complete where they overlap with the bullets above.

## Phase 0: Foundation & MVP (Months 1-2)

**Goal:** Build a working prototype that demonstrates core value proposition

### Core Components
- [x] **CLI Tool (Rust)** (local POC)
  - [x] Basic command structure (`init`, `status`, `search`, `add`, `vault`, `changelog`, `hooks`, `automation`, `mcp`, `sync` stub)
  - [x] Configuration management (`.joicy/joicy.toml`, legacy `config.toml`)
  - [x] Local SQLite + FTS5 storage (default)

- [ ] **Local Memory Bank** (full vision)
  - [ ] Embed code snippets using local embedding model
  - [ ] Vector storage (Qdrant/Chroma; stubs only today)
  - [x] Basic **full-text** search (FTS5)
  - [x] Metadata indexing (file path, language, timestamp)

- [x] **Git Integration** (local POC)
  - [x] **Post-commit** hook installation (`joicy hooks install`)
  - [x] Commit metadata + capture pipeline (`git` CLI)
  - [x] Automatic storage on commit (SQLite + vault + changelog + ticket stub)
  - [ ] Pattern detection (similarity beyond FTS)

- [x] **MCP Server (Basic)** (local POC)
  - [x] MCP protocol (rmcp)
  - [x] Memory bank query / store tools
  - [x] Changelog + vault note tools
  - [ ] Deeper IDE integration (file-scoped context, etc.)

### Success Criteria
- Developer can initialize memory bank
- Code context is stored on commit
- AI agent can query memory bank via MCP
- Basic pattern matching works

---

## Phase 1: Core Features (Months 3-4)

**Goal:** Production-ready individual developer experience

### Enhanced Storage
- [ ] **Vector Database Integration**
  - [ ] Qdrant integration (or Chroma for simplicity)
  - [ ] Local Qdrant instance management
  - [ ] Migration from SQLite to Qdrant
  - [ ] Index optimization

- [ ] **Caching Layer**
  - [ ] Hot cache implementation (in-memory)
  - [ ] Cache invalidation strategy
  - [ ] Performance metrics

### CLI Enhancements
- [ ] **Search Commands**
  - [ ] `joicy search <query>` - semantic search
  - [ ] `joicy search --file <path>` - file-specific search
  - [ ] `joicy search --pattern <pattern>` - pattern search
  - [ ] Search result formatting and ranking

- [ ] **Management Commands**
  - [ ] `joicy sync` - manual sync trigger
  - [ ] `joicy status` - show memory bank stats
  - [ ] `joicy clean` - cleanup old entries
  - [ ] `joicy export` - export memory bank

### Git Integration Improvements
- [ ] **Advanced Pattern Detection**
  - [ ] Bug pattern recognition
  - [ ] Code smell detection
  - [ ] Solution matching
  - [ ] Confidence scoring

- [ ] **Commit Analysis**
  - [ ] Commit message extraction
  - [ ] Feature tagging
  - [ ] Related commit linking

### MCP Server Enhancements
- [ ] **Advanced Tools**
  - [ ] Context retrieval for current file
  - [ ] Similar code search
  - [ ] Solution suggestions
  - [ ] Pattern explanation

### Documentation
- [ ] User guide
- [ ] Installation instructions
- [ ] Configuration examples
- [ ] Troubleshooting guide

### Success Criteria
- Fast search (<300ms for local queries)
- Accurate pattern matching
- Smooth MCP integration
- Production-ready for individual use

---

## Phase 2: Team Features (Months 5-7)

**Goal:** Enable team knowledge sharing

### Central Memory Bank
- [ ] **Backend API Service**
  - [ ] REST API design
  - [ ] Authentication (API keys, OAuth)
  - [ ] Multi-tenant support
  - [ ] Rate limiting

- [ ] **Central Vector Database**
  - [ ] Central Qdrant instance
  - [ ] Team-level collections
  - [ ] Access control per team
  - [ ] Data isolation

### Sync Service
- [ ] **Background Sync**
  - [ ] Async sync queue
  - [ ] Conflict resolution
  - [ ] Incremental sync
  - [ ] Sync status tracking

- [ ] **Sync Strategies**
  - [ ] Push: Local → Central
  - [ ] Pull: Central → Local
  - [ ] Bidirectional sync
  - [ ] Selective sync (by team/project)

### Team Management
- [ ] **CLI Team Commands**
  - [ ] `joicy team create <name>`
  - [ ] `joicy team join <team-id>`
  - [ ] `joicy team list`
  - [ ] `joicy team members`

- [ ] **Access Control**
  - [ ] Team roles (admin, member, viewer)
  - [ ] Permission management
  - [ ] Team-level privacy settings

### Enhanced Search
- [ ] **Cross-Team Search**
  - [ ] Search local + team memory
  - [ ] Search company-wide (if permitted)
  - [ ] Result source attribution
  - [ ] Team filter in search

### Web Dashboard (Optional)
- [ ] Basic web UI for team management
- [ ] Memory bank statistics
- [ ] Team activity dashboard
- [ ] Search interface

### Success Criteria
- Teams can share knowledge
- Sync works reliably
- Access control enforced
- 10+ developers can use simultaneously

---

## Phase 3: Enterprise Features (Months 8-10)

**Goal:** Support large organizations and air-gapped deployments

### Deployment Options
- [ ] **Self-Hosted Deployment**
  - [ ] Docker Compose setup
  - [ ] Kubernetes Helm charts
  - [ ] Installation scripts
  - [ ] Configuration management

- [ ] **Air-Gapped Support**
  - [ ] Offline installation
  - [ ] Manual sync mechanisms
  - [ ] Export/import functionality
  - [ ] Network isolation handling

- [ ] **SaaS Option**
  - [ ] Cloud infrastructure setup
  - [ ] Multi-region support
  - [ ] Auto-scaling
  - [ ] High availability

### Enterprise Features
- [ ] **Advanced Authentication**
  - [ ] SAML SSO
  - [ ] LDAP/AD integration
  - [ ] OAuth providers
  - [ ] MFA support

- [ ] **Audit & Compliance**
  - [ ] Audit logging
  - [ ] Data retention policies
  - [ ] GDPR compliance features
  - [ ] Export for compliance

- [ ] **Administration**
  - [ ] Admin dashboard
  - [ ] User management
  - [ ] Team management UI
  - [ ] System monitoring

### Performance & Scale
- [ ] **Optimization**
  - [ ] Query optimization
  - [ ] Index tuning
  - [ ] Caching improvements
  - [ ] Load testing

- [ ] **Scalability**
  - [ ] Horizontal scaling
  - [ ] Database sharding
  - [ ] CDN integration
  - [ ] Distributed caching

### VSCode Extension
- [ ] **Extension Development**
  - [ ] Extension scaffolding
  - [ ] Memory bank panel
  - [ ] Inline suggestions
  - [ ] Real-time updates

- [ ] **Integration Features**
  - [ ] Git hook UI
  - [ ] Search from editor
  - [ ] Pattern alerts
  - [ ] Team activity feed

### Success Criteria
- Supports 100+ developers
- Air-gapped deployment works
- Enterprise authentication integrated
- VSCode extension published

---

## Phase 4: Advanced Features (Months 11-12+)

**Goal:** Advanced AI capabilities and ecosystem integration

### Advanced AI Features
- [ ] **Intelligent Analysis**
  - [ ] Code quality scoring
  - [ ] Technical debt detection
  - [ ] Refactoring suggestions
  - [ ] Architecture pattern recognition

- [ ] **Predictive Features**
  - [ ] Bug prediction
  - [ ] Code review suggestions
  - [ ] Test coverage recommendations
  - [ ] Performance optimization hints

### CI/CD Integration
- [ ] **GitHub Actions**
  - [ ] Action for PR analysis
  - [ ] Automated pattern detection
  - [ ] Review comment generation

- [ ] **GitLab CI**
  - [ ] Pipeline integration
  - [ ] MR analysis
  - [ ] Automated suggestions

- [ ] **Other Platforms**
  - [ ] Bitbucket integration
  - [ ] Azure DevOps
  - [ ] Generic webhook support

### Documentation Generation
- [ ] **Auto-Documentation**
  - [ ] Generate docs from memory bank
  - [ ] API documentation
  - [ ] Architecture diagrams
  - [ ] Best practices guide

### Analytics & Insights
- [ ] **Team Analytics**
  - [ ] Knowledge distribution
  - [ ] Pattern frequency
  - [ ] Bug recurrence tracking
  - [ ] Team productivity metrics

- [ ] **Insights Dashboard**
  - [ ] Visualization of patterns
  - [ ] Trend analysis
  - [ ] Recommendations
  - [ ] Reports generation

### Ecosystem Integration
- [ ] **IDE Support**
  - [ ] JetBrains plugins
  - [ ] Neovim integration
  - [ ] Emacs support

- [ ] **API Ecosystem**
  - [ ] Public API
  - [ ] Webhooks
  - [ ] SDK development
  - [ ] Third-party integrations

### Success Criteria
- Advanced AI features working
- CI/CD integrations complete
- Analytics provide value
- Ecosystem integrations available

---

## Future Considerations

### Research Areas
- [ ] Multi-language code understanding
- [ ] Cross-repository pattern detection
- [ ] Automated test generation from patterns
- [ ] Code generation from memory bank

### Potential Features
- [ ] Mobile app for memory bank access
- [ ] Slack/Teams integration
- [ ] Knowledge base export to Confluence/Notion
- [ ] AI model fine-tuning on team code

---

## Milestones Summary

| Phase | Duration | Key Deliverable | Target Users |
|-------|----------|----------------|--------------|
| Phase 0 | 2 months | MVP with local memory bank | Individual developers |
| Phase 1 | 2 months | Production-ready CLI + MCP | Individual developers |
| Phase 2 | 3 months | Team sync and sharing | Small teams (5-20 devs) |
| Phase 3 | 3 months | Enterprise deployment | Large teams (20-100+ devs) |
| Phase 4 | 2+ months | Advanced AI features | Enterprise + ecosystem |

---

## Success Metrics

### Phase 0-1 (Individual)
- Memory bank initialization time < 5s
- Search latency < 300ms
- Pattern detection accuracy > 80%
- MCP integration success rate > 95%

### Phase 2 (Team)
- Sync latency < 5s
- Sync reliability > 99%
- Team member onboarding < 2 minutes
- Cross-team search accuracy > 75%

### Phase 3 (Enterprise)
- Support 100+ concurrent users
- Uptime > 99.9%
- Authentication integration < 1 day setup
- Air-gapped deployment < 4 hours

### Phase 4 (Advanced)
- CI/CD integration adoption > 50%
- Analytics usage > 30% of teams
- Third-party integrations > 5
- User satisfaction > 4.5/5

---

## Notes

- Phases may overlap based on team capacity
- Priorities may shift based on user feedback
- Some features may be moved between phases
