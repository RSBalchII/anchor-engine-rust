# Documentation Policy

This document outlines the policy for maintaining documentation in the Anchor Engine Rust project.

## Allowed Documentation Locations

### ✅ PERMITTED LOCATIONS

#### User-Facing Documentation
```
docs/
├── setup/              # Installation and configuration guides
├── api/                # API reference documentation
├── guides/             # How-to guides and tutorials
├── troubleshooting/    # Problem-solving guides
├── technical/          # Deep-dive technical documentation
└── INDEX.md            # Documentation navigation index
```

#### Technical Specifications
```
specs/
├── current-standards/  # Active technical standards (001-XXX)
├── proposals/          # Proposed standards and RFCs
└── archive/            # Historical and deprecated standards
```

#### Package-Level Documentation
```
[PACKAGE]/README.md     # Package-specific documentation only
e.g., crates/anchor-mcp/README.md
     crates/anchor-ui/README.md
```

### ❌ FORBIDDEN LOCATIONS

**Never create documentation in:**
- Root directory (except README.md, CHANGELOG.md, LICENSE)
- `benchmarks/*.md`
- `tests/*.md`
- `scripts/*.md`
- `engine/*.md` (subdirectories)
- Hidden directories (`.ai/`, `.cursor/`, `.jules/`)
- Any other location not explicitly permitted

## Documentation Principles

### 1. Single Source of Truth
- No duplicate content across multiple files
- Link to existing documentation instead of copying
- Update existing docs rather than creating new ones

### 2. Actionable Content
- Documentation must be instructional
- Include code examples where relevant
- Provide clear next steps
- Avoid philosophical discussions

### 3. Concise and Clear
- No unnecessary verbosity
- Use clear, simple language
- Include relevant examples
- Focus on practical utility

## Before Creating Documentation

### Checklist
Before creating any documentation, verify:

1. [ ] Does this content already exist in `docs/` or `specs/`?
2. [ ] Is this the correct location (docs/ vs specs/)?
3. [ ] Is this instructional (not philosophical)?
4. [ ] Is this concise (not verbose)?
5. [ ] Am I creating this in an allowed directory?

### Search Commands
```bash
# Search for existing content
grep -r "your topic" docs/ specs/

# Check recent commits for similar topics
git log --oneline -50 | grep -i "topic"
```

## Maintenance Responsibilities

### Regular Audits
- Quarterly review of documentation structure
- Consolidate duplicate content
- Archive outdated documentation
- Update cross-references

### Quality Checks
- Verify all links are valid
- Ensure examples are up-to-date
- Confirm instructions work as described
- Check for consistency with current implementation

## Enforcement

### For AI Assistants
- Read this policy before creating any documentation
- Verify location is allowed before creating files
- Search for existing content before writing
- Follow the checklist before creating new docs

### For Human Contributors
- Pre-commit hooks check for forbidden locations
- PR templates include documentation location verification
- Code reviews verify documentation placement

### For CI/CD
- Fail builds if .md files exist in forbidden directories
- Warn if standards are older than 6 months without updates
- Verify docs/INDEX.md links are valid

## Related Standards

- [Standard 022: Documentation Hygiene](specs/current-standards/022-documentation-hygiene.md) - Detailed hygiene requirements
- [Architecture Spec](specs/current-standards/001-architecture-spec.md) - System architecture documentation

---

**Policy Version:** 1.0  
**Last Updated:** 2026-03-31  
**Owner:** Project Maintainers  
**Enforcement:** Mandatory for all contributors