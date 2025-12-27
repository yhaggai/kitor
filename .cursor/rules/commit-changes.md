---
description: 
globs: 
alwaysApply: false
---
# Commit Staged Changes with Gitmoji

A comprehensive guide for creating meaningful commits with proper descriptions and appropriate gitmoji emojis for **staged files only**.

## Important - linear integration rules 

### **Ticket Identification**
At least one commit in branch must contain inside the commit message the keyword that identifies the Linear ticket.
The keyword is derived from the branch name.

**Example:**
- Branch: `dave/mobb-3046-implement-the-fe-part-of-the-report`
- Commit message must contain: `Fix MOBB-3046` (or `MOBB-3046`)

### **Reading Linear Ticket Description**

Before creating a commit message, **always fetch and review the Linear ticket description** to ensure the commit message accurately reflects the ticket's requirements and context.

#### **Workflow:**

1. **Extract Ticket ID from Branch Name**
   ```bash
   # Get current branch name
   git branch --show-current
   
   # Extract ticket ID (e.g., MOBB-3046 from dave/mobb-3046-implement-the-fe-part-of-the-report)
   # Pattern: <team-prefix>-<number> (case-insensitive)
   ```

2. **Fetch Linear Ticket Details**
   - Use Linear MCP tools to retrieve ticket information:
     - `mcp_user-Linear_get_issue` - Get full ticket details including description
     - Extract ticket ID from branch name (e.g., `MOBB-3046` from `dave/mobb-3046-...`)
   
3. **Review Ticket Description**
   - Read the ticket title and description
   - Understand the requirements and context
   - Note any specific implementation details or acceptance criteria

4. **Incorporate into Commit Message**
   - Reference the ticket ID in the commit message
   - Align commit description with ticket requirements
   - Use ticket context to write accurate commit body

## Important: Staged Files Only

This rule focuses on committing **only staged/added files**. Always review and stage specific changes before committing.

```bash
# Check status to see staged vs unstaged files
git status

# Stage specific files you want to commit
git add <specific-files>

# Commit only the staged changes with summary in body
git commit -m "<gitmoji> <type>: <description>" -m "<short summary of changes>"

# Or use multi-line commit for longer messages
git commit
```

## Commit Message Format

```
<gitmoji> <type>: <description>

<short summary of what was changed, why it was changed, and impact>

[optional footer]
```

### **Format Guidelines:**
- **Subject Line**: Keep under 50 characters with gitmoji, type, and brief description
- **Body (Required)**: 2-3 sentences summarizing:
  - What was changed
  - Why it was changed  
  - Key impact or benefits
- **Footer**: Optional for breaking changes, issue references, etc.

### **Body Summary Examples:**
```
✨ feat: add user authentication with JWT tokens

Implemented secure login/logout system using JWT tokens and NextAuth.js.
Added protected routes middleware and user session management.
Users can now securely access personalized content and maintain login state.
```

🐛 fix: resolve memory leak in image processing

Fixed memory leak caused by unclosed file streams in image resize utility.
Updated garbage collection and added proper resource cleanup.
Reduces memory usage by ~40MB per image processing operation.
```

## Common Gitmoji Categories

### 🎯 **New Features & Improvements**
- ✨ `:sparkles:` - Introduce new features
- 🎨 `:art:` - Improve structure/format of code
- ⚡️ `:zap:` - Improve performance
- 💄 `:lipstick:` - Add or update UI and style files
- ♿️ `:wheelchair:` - Improve accessibility
- 🚸 `:children_crossing:` - Improve user experience/usability

### 🐛 **Bug Fixes & Hotfixes**
- 🐛 `:bug:` - Fix a bug
- 🚑️ `:ambulance:` - Critical hotfix
- 🩹 `:adhesive_bandage:` - Simple fix for non-critical issue
- 🔒️ `:lock:` - Fix security or privacy issues

### 📝 **Documentation & Comments**
- 📝 `:memo:` - Add or update documentation
- 💡 `:bulb:` - Add or update comments in source code
- 📄 `:page_facing_up:` - Add or update license

### 🧪 **Testing**
- ✅ `:white_check_mark:` - Add, update, or pass tests
- 🧪 `:test_tube:` - Add a failing test
- 🤡 `:clown_face:` - Mock things

### 🔧 **Configuration & Dependencies**
- 🔧 `:wrench:` - Add or update configuration files
- 🔨 `:hammer:` - Add or update development scripts
- ➕ `:heavy_plus_sign:` - Add a dependency
- ➖ `:heavy_minus_sign:` - Remove a dependency
- ⬆️ `:arrow_up:` - Upgrade dependencies
- ⬇️ `:arrow_down:` - Downgrade dependencies
- 📌 `:pushpin:` - Pin dependencies to specific versions

### 🏗️ **Code Structure & Refactoring**
- ♻️ `:recycle:` - Refactor code
- 🚚 `:truck:` - Move or rename resources
- 🔥 `:fire:` - Remove code or files
- ⚰️ `:coffin:` - Remove dead code
- 🏗️ `:building_construction:` - Make architectural changes

### 🚀 **Deployment & CI/CD**
- 🚀 `:rocket:` - Deploy stuff
- 👷 `:construction_worker:` - Add or update CI build system
- 💚 `:green_heart:` - Fix CI Build
- 🔖 `:bookmark:` - Release/Version tags

### 🗃️ **Data & Database**
- 🗃️ `:card_file_box:` - Perform database related changes
- 🌱 `:seedling:` - Add or update seed files
- 🧐 `:monocle_face:` - Data exploration/inspection

### 🔍 **Maintenance & Cleanup**
- 🚨 `:rotating_light:` - Fix compiler/linter warnings
- ✏️ `:pencil2:` - Fix typos
- 🙈 `:see_no_evil:` - Add or update .gitignore file
- 🗑️ `:wastebasket:` - Deprecate code that needs cleanup

## Commit Description Guidelines

### ✅ **Good Commit Messages**
```bash
✨ feat: add user authentication with JWT tokens

Implemented secure login/logout system using JWT tokens and NextAuth.js.
Added protected routes middleware and user session management.
Users can now securely access personalized content and maintain login state.

🐛 fix: resolve memory leak in image processing

Fixed memory leak caused by unclosed file streams in image resize utility.
Updated garbage collection and added proper resource cleanup.
Reduces memory usage by ~40MB per image processing operation.

🎨 refactor: reorganize component file structure

Restructured components into feature-based directories for better maintainability.
Moved shared components to common folder and updated import paths.
Improves developer experience and reduces bundle size through better tree-shaking.

📝 docs: update API documentation for v2.0

Updated all endpoint documentation with new request/response schemas.
Added authentication examples and error handling guidelines.
Provides clear guidance for frontend developers integrating with the API.

🔧 config: add TypeScript strict mode configuration

Enabled strict type checking and null safety in TypeScript config.
Updated all components to pass strict type validation.
Catches more potential runtime errors during development phase.
```

### ❌ **Bad Commit Messages**
```bash
fix stuff
update
changes
wip
asdf
```

## Writing Effective Descriptions

1. **Use imperative mood**: "add feature" not "added feature"
2. **Be specific**: Explain what changed and why
3. **Keep first line under 50 characters**
4. **Always include body summary**: 2-3 sentences explaining the change
5. **Reference issues/tickets when applicable**

## Examples by Category

### Feature Development
```bash
git commit -m "✨ feat: implement real-time chat functionality" \
           -m "Added WebSocket connection with message broadcasting and user presence.
               Integrated with existing user authentication for secure messaging.
               Enables instant communication between users with typing indicators."

git commit -m "💄 style: redesign dashboard with modern card layout" \
           -m "Replaced table-based layout with responsive card grid design.
               Added dark mode support and improved mobile responsiveness.
               Enhances user experience with cleaner visual hierarchy."

git commit -m "🚸 ux: add loading states for better user feedback" \
           -m "Implemented skeleton loaders and progress indicators throughout app.
               Added error boundaries with retry mechanisms for failed requests.
               Reduces perceived loading time and improves user satisfaction."
```

### Bug Fixes
```bash
git commit -m "🐛 fix: prevent duplicate form submissions" \
           -m "Added debouncing and disabled button state during form processing.
               Fixed race condition that allowed multiple simultaneous submissions.
               Prevents data corruption and improves form reliability."

git commit -m "🚑️ hotfix: resolve critical payment processing error" \
           -m "Fixed payment gateway timeout causing transaction failures.
               Updated error handling and retry logic for failed payments.
               Restores payment functionality for all users immediately."

git commit -m "🩹 patch: fix minor styling issue in mobile navigation" \
           -m "Corrected menu overlay z-index conflict with modal components.
               Adjusted responsive breakpoints for better mobile experience.
               Ensures navigation remains accessible on all screen sizes."
```

### Code Quality
```bash
git commit -m "♻️ refactor: extract reusable validation logic" \
           -m "Created centralized validation utilities for forms and API inputs.
               Removed duplicated validation code across 15+ components.
               Improves maintainability and ensures consistent validation rules."

git commit -m "🎨 style: format code according to new linting rules" \
           -m "Applied Prettier and ESLint configurations across entire codebase.
               Updated import ordering and removed unused variables.
               Ensures consistent code style and improved readability."

git commit -m "⚰️ cleanup: remove deprecated API endpoints" \
           -m "Removed legacy v1 API endpoints and related middleware.
               Updated documentation to reflect v2 API usage only.
               Reduces codebase complexity and eliminates security vulnerabilities."
```

### Dependencies & Configuration
```bash
git commit -m "⬆️ deps: upgrade React to v18.2.0" \
           -m "Updated React and related packages to latest stable versions.
               Migrated deprecated lifecycle methods and updated testing utilities.
               Provides performance improvements and access to concurrent features."

git commit -m "🔧 config: add environment variables for staging" \
           -m "Added staging-specific environment configuration and deployment scripts.
               Separated staging and production database connections.
               Enables safe testing of features before production deployment."

git commit -m "➕ deps: add lodash for utility functions" \
           -m "Integrated lodash for consistent utility functions across the app.
               Replaced custom implementations with battle-tested library methods.
               Reduces bundle size and improves code reliability."
```

## Pre-Commit Workflow

### 1. **Review Current Status**
```bash
# See what files are staged, unstaged, and untracked
git status

# Review staged changes
git diff --cached

# Review unstaged changes (don't commit these)
git diff
```

### 2. **Stage Specific Changes**
```bash
# Stage specific files
git add src/components/Button.tsx
git add src/utils/helpers.ts

# Stage parts of a file (interactive)
git add -p <filename>

# Stage all changes in a directory
git add src/components/
```

### 3. **Verify Staged Changes**
```bash
# Review what will be committed
git diff --cached

# See list of staged files
git diff --cached --name-only
```

### 4. **Commit Staged Changes Only**
```bash
# Commit with gitmoji and description
git commit -m "✨ feat: add reusable Button component with variants"

# For longer commit messages
git commit
```

## Workflow Integration

### ✅ **Recommended Workflow**
```bash
# 1. Check current status
git status

# 2. Stage specific files you want to commit
git add src/components/NewFeature.tsx
git add src/types/api.ts

# 3. Review staged changes
git diff --cached

# 4. Commit only staged files
git commit -m "✨ feat: implement user profile API integration"

# 5. When working on unstaged files ask first if the user wants to continue committing them
```

### ❌ **Avoid These Commands**
```bash
# DON'T commit all changes indiscriminately
git add .
git commit -m "update stuff"

# DON'T use commit -a (commits all tracked files)
git commit -a -m "changes"

# DON'T commit without reviewing staged changes
git commit -m "fix" # without checking what's staged
```

## Staging Strategies

### **File-Level Staging**
```bash
# Stage complete files
git add src/components/Header.tsx
git add src/styles/globals.css
```

### **Partial Staging (Hunks)**
```bash
# Stage parts of a file interactively
git add -p src/utils/api.ts

# Options during interactive staging:
# y - stage this hunk
# n - don't stage this hunk
# s - split into smaller hunks
# q - quit
```

### **Directory Staging**
```bash
# Stage all files in a directory
git add src/components/
git add tests/unit/
```

## Status Checking Commands

```bash
# Detailed status
git status

# Short status format
git status -s

# See staged changes
git diff --cached

# See unstaged changes
git diff

# List staged files only
git diff --cached --name-only

# List unstaged files only
git diff --name-only
```

## Examples by Staging Scenario

### **New Feature Development**
```bash
# Stage new component and its test
git add src/components/UserProfile.tsx
git add src/components/UserProfile.test.tsx
git commit -m "✨ feat: add UserProfile component with avatar display"

# Continue working on styling (don't commit yet)
# Edit src/styles/profile.css (leave unstaged)
```

### **Bug Fix**
```bash
# Stage only the bug fix, not other WIP changes
git add src/utils/validation.ts
git commit -m "🐛 fix: resolve email validation regex issue"

# Keep other experimental changes unstaged
```

### **Documentation Update**
```bash
# Stage only documentation changes
git add README.md
git add docs/api.md
git commit -m "📝 docs: update API documentation for v2.0"

# Keep code changes for separate commit
```

## Quick Reference Questions

Before committing, ask yourself:
1. **Have I staged only the changes I want to commit?**
2. **Are there unstaged changes that should remain uncommitted?**
3. **What emoji best represents the staged changes?**
4. **Can someone understand what changed from the description?**
5. **Should I split this into multiple smaller commits?**

## Commit Summary with Details

After successfully committing staged changes, provide a comprehensive summary following this format:

### ✅ **Commit Summary Template**

```
## ✅ Commit Summary

**Commit Message:** `<gitmoji> <type>: <description>`

**What was committed:**
- <gitmoji> **<Category>**: <Brief description of changes>
- <gitmoji> **<Category>**: <Brief description of changes>

**Files Changed:** X files with Y insertions and Z deletions

**Key Changes:**
- **New Files**: List any new files created
- **Modified Files**: Highlight important file modifications
- **Dependencies**: Note any package/dependency changes
- **Configuration**: Mention config or setup changes
- **Features**: Describe new functionality added
- **Bug Fixes**: Detail what issues were resolved
- **Refactoring**: Explain code improvements made

**Impact:**
- **User-Facing**: How this affects end users
- **Developer Experience**: How this affects development workflow
- **Performance**: Any performance implications
- **Security**: Security-related improvements or considerations

**Next Steps:**
- List any follow-up tasks or related work needed
- Mention any testing that should be done
- Note any deployment considerations

**Repository Status:**
- Clean working directory ✅
- All staged changes committed ✅
- Ready for next development phase ✅
```

## ✅ Commit Summary

**Commit Message:** `✨ feat: implement homepage with hero section`

**What was committed:**
- ✨ **New Feature**: Homepage with hero section
- 💄 **UI Components**: Added Badge, Button, Card, Separator

**Files Changed:** 11 files | +510 / -104 lines

**Highlights:**
- **New**: 4 Shadcn components
- **Updated**: `page.tsx`, `layout.tsx`
- **Dependencies**: Added UI library
- **Features**: Responsive layout with value props

**Impact:**
- **Users**: See new landing page
- **Devs**: Gain reusable components
- **SEO**: Improved structure

**Next Steps:**
- Test responsiveness
- Integrate auth
- Add analytics

### 🎨 **Visual Summary Format**

For enhanced readability, use this visual format:

```
🎉 **COMMIT SUCCESSFUL** 🎉

┌─ Commit Details ─────────────────────────────────────┐
│ Message: ✨ feat: implement user authentication      │
│ Hash: a1b2c3d                                        │
│ Files: 8 changed (+156 -23)                         │
└──────────────────────────────────────────────────────┘

📦 **CHANGES SUMMARY**
├── ✨ New Features
│   ├── JWT authentication system
│   ├── Login/logout components
│   └── Protected route middleware
├── 🔧 Configuration
│   ├── NextAuth.js setup
│   ├── Environment variables
│   └── Database schema updates
└── 📝 Documentation
    ├── API documentation
    └── Setup instructions

🎯 **IMPACT ANALYSIS**
├── 👥 Users: Secure login experience
├── 🛠️ Developers: Auth utilities available
├── 🚀 Performance: Minimal impact
└── 🔒 Security: Enhanced protection

📋 **NEXT ACTIONS**
├── [ ] Test OAuth providers
├── [ ] Update user dashboard
├── [ ] Deploy to staging
└── [ ] Update documentation
```

### 🏆 **Best Practices for Commit Summaries**

1. **Be Comprehensive but Concise**
   - Include all relevant information
   - Avoid unnecessary verbosity
   - Focus on impact and changes

2. **Use Consistent Formatting**
   - Follow the template structure
   - Use emojis for visual clarity
   - Maintain consistent categorization

3. **Include Context**
   - Explain why changes were made
   - Reference related issues or PRs
   - Mention any breaking changes

4. **Document Dependencies**
   - List new packages and versions
   - Explain dependency choices
   - Note any compatibility requirements

5. **Plan Next Steps**
   - Identify follow-up tasks
   - Mention testing requirements
   - Note deployment considerations

6. **Track Progress**
   - Reference completed tasks/tickets
   - Update project status
   - Communicate with team members

## Useful Aliases

Add these to your `.gitconfig` for easier staging workflows:

```bash
[alias]
    st = status
    staged = diff --cached
    unstaged = diff
    stage = add
    unstage = reset HEAD --
    review = diff --cached --name-status
```

Remember: **Only commit staged changes** to maintain clean, intentional commits. Keep unstaged and untracked files for future commits or experimentation.