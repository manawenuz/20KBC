#!/usr/bin/env node
/**
 * Reads .taskmaster/tasks/tasks.json and generates a Docusaurus-compatible
 * markdown page at website/docs/tasks.md
 */

import { readFileSync, writeFileSync } from 'fs';
import { resolve, dirname } from 'path';
import { fileURLToPath } from 'url';

const __dirname = dirname(fileURLToPath(import.meta.url));
const root = resolve(__dirname, '..');
const tasksFile = resolve(root, '.taskmaster/tasks/tasks.json');
const outputFile = resolve(root, 'website/docs/tasks.md');

const data = JSON.parse(readFileSync(tasksFile, 'utf-8'));
const tasks = data.master?.tasks || [];

// Stats
const total = tasks.length;
const done = tasks.filter(t => t.status === 'done').length;
const inProgress = tasks.filter(t => t.status === 'in-progress').length;
const pending = tasks.filter(t => t.status === 'pending').length;
const pct = total ? Math.round((done / total) * 100) : 0;

// Progress bar
const barLen = 30;
const filled = Math.round((done / total) * barLen);
const bar = '█'.repeat(filled) + '░'.repeat(barLen - filled);

// Status icons
function icon(status) {
  switch (status) {
    case 'done': return '✅';
    case 'in-progress': return '🔄';
    case 'pending': return '⬚';
    case 'blocked': return '🚫';
    case 'cancelled': return '❌';
    case 'deferred': return '⏸️';
    case 'review': return '👀';
    default: return '○';
  }
}

function priorityBadge(p) {
  switch (p) {
    case 'high': return '🔴';
    case 'medium': return '🟡';
    case 'low': return '🟢';
    default: return '';
  }
}

// Group by phase tag
function getPhase(task) {
  const tags = task.tags || [];
  if (tags.includes('phase-0')) return 'Phase 0: Technology Selection';
  if (tags.includes('phase-1')) return 'Phase 1: MVP Prototype';
  return 'Cross-Phase';
}

const phases = {};
for (const t of tasks) {
  const phase = getPhase(t);
  if (!phases[phase]) phases[phase] = [];
  phases[phase].push(t);
}

// Build markdown
let md = `---
sidebar_position: 99
---

# Project Tasks

> Auto-generated from \`.taskmaster/tasks/tasks.json\`
> Last updated: ${new Date().toISOString().split('T')[0]}

## Progress

\`\`\`
${bar} ${pct}% (${done}/${total})
\`\`\`

| Status | Count |
|--------|-------|
| ✅ Done | ${done} |
| 🔄 In Progress | ${inProgress} |
| ⬚ Pending | ${pending} |
| **Total** | **${total}** |

---

`;

const phaseOrder = ['Phase 0: Technology Selection', 'Phase 1: MVP Prototype', 'Cross-Phase'];

for (const phase of phaseOrder) {
  const phaseTasks = phases[phase];
  if (!phaseTasks || phaseTasks.length === 0) continue;

  const pDone = phaseTasks.filter(t => t.status === 'done').length;
  const pTotal = phaseTasks.length;

  md += `## ${phase} (${pDone}/${pTotal})\n\n`;
  md += `| | ID | Task | Status | Priority | Depends On |\n`;
  md += `|---|---|------|--------|----------|------------|\n`;

  for (const t of phaseTasks) {
    const deps = t.dependencies?.length ? t.dependencies.join(', ') : '—';
    md += `| ${icon(t.status)} | ${t.id} | **${t.title}** | ${t.status} | ${priorityBadge(t.priority)} ${t.priority} | ${deps} |\n`;
  }

  md += '\n';

  // Expandable subtasks
  for (const t of phaseTasks) {
    if (t.subtasks && t.subtasks.length > 0) {
      const stDone = t.subtasks.filter(s => s.status === 'done').length;
      md += `<details>\n<summary><strong>#${t.id} ${t.title}</strong> — subtasks (${stDone}/${t.subtasks.length})</summary>\n\n`;
      md += `| | Subtask | Status |\n`;
      md += `|---|---------|--------|\n`;
      for (const s of t.subtasks) {
        md += `| ${icon(s.status)} | ${s.title} | ${s.status} |\n`;
      }
      md += `\n</details>\n\n`;
    }
  }

  md += '---\n\n';
}

writeFileSync(outputFile, md);
console.log(`Generated ${outputFile} (${total} tasks, ${done} done)`);
