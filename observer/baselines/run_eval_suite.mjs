#!/usr/bin/env node
// run_eval_suite.mjs — Eval tuning test harness for Valhalla
//
// Feeds tactical positions from tactical_samples.json to the engine,
// compares bestmove against human reference, reports score.
//
// Usage:
//   node run_eval_suite.mjs <path-to-valhalla-binary>
//
// Example:
//   node run_eval_suite.mjs ../../target/release/valhalla-engine

import { readFileSync } from 'node:fs';
import { spawn } from 'node:child_process';
import { createInterface } from 'node:readline';
import { fileURLToPath } from 'node:url';
import { dirname, join } from 'node:path';

const __dirname = dirname(fileURLToPath(import.meta.url));
const SAMPLES_PATH = join(__dirname, 'tactical_samples.json');

// ── Engine Communication ──

class Engine {
  #proc;
  #rl;
  #lineQueue = [];
  #lineResolve = null;
  #dead = false;

  constructor(enginePath) {
    this.#proc = spawn(enginePath, [], { stdio: ['pipe', 'pipe', 'pipe'] });
    this.#proc.on('exit', () => { this.#dead = true; });
    this.#proc.stderr.on('data', () => {}); // swallow stderr
    this.#rl = createInterface({ input: this.#proc.stdout });
    this.#rl.on('line', (line) => {
      if (this.#lineResolve) {
        const r = this.#lineResolve;
        this.#lineResolve = null;
        r(line);
      } else {
        this.#lineQueue.push(line);
      }
    });
  }

  send(cmd) {
    if (!this.#dead) this.#proc.stdin.write(cmd + '\n');
  }

  async readLine() {
    if (this.#lineQueue.length > 0) return this.#lineQueue.shift();
    if (this.#dead) return null;
    return new Promise((resolve) => { this.#lineResolve = resolve; });
  }

  async readUntil(prefix) {
    const lines = [];
    while (true) {
      const line = await this.readLine();
      if (line === null) break;
      lines.push(line);
      if (line.startsWith(prefix)) break;
    }
    return lines;
  }

  quit() { this.send('quit'); }
}

// ── Notation Conversion ──
// chess.com 4PC: "Ne1-f3", "Qg1xj4", "O-O", "h7-h8=D"
// Valhalla:      "e1f3",   "g1j4",   "h1j1", "h7h8q"

function chessComToValhalla(move, player) {
  if (!move || move === 'T' || move === 'S' || move === 'R') return null;

  // Castling — king from/to squares for each player on 14x14 board
  if (move === 'O-O' || move === 'O-O-O') {
    const castleMap = {
      'Red':    { 'O-O': 'h1j1',  'O-O-O': 'h1f1'  },
      'Blue':   { 'O-O': 'a8a10', 'O-O-O': 'a8a6'  },
      'Yellow': { 'O-O': 'h14f14','O-O-O': 'h14j14' },
      'Green':  { 'O-O': 'n8n10', 'O-O-O': 'n8n6'  },
    };
    return castleMap[player]?.[move] || null;
  }

  // Strip piece prefix (N, B, R, Q, K)
  let m = move.replace(/^[NBRQK]/, '');

  // Handle captures: e5xRc4 → e5c4
  m = m.replace(/x[NBRQK]?/, '');

  // Handle promotion: h8=D → h8q
  m = m.replace(/=D/, 'q').replace(/=Q/, 'q').replace(/=R/, 'r').replace(/=B/, 'b').replace(/=N/, 'n');

  // Remove dashes: e1-f3 → e1f3
  m = m.replace(/-/g, '');

  // Remove check/checkmate markers
  m = m.replace(/[+#]/g, '');

  // Convert to lowercase
  return m.toLowerCase();
}

// ── Move Category Detection ──

function categorizeMove(moveStr) {
  if (!moveStr) return 'unknown';
  if (moveStr.includes('x') || moveStr.includes('X')) return 'capture';
  if (moveStr === 'O-O' || moveStr === 'O-O-O') return 'castling';
  if (moveStr.includes('=')) return 'promotion';
  if (moveStr.includes('#')) return 'checkmate';
  if (moveStr.includes('+')) return 'check';
  if (/^[NBRQK]/.test(moveStr)) return 'piece_move';
  // Pawn move (no piece prefix)
  return 'pawn_move';
}

function categorizeExpected(sample) {
  const cat = sample.expected_category;
  const catMap = {
    'capture': 'capture',
    'sacrifice': 'capture',
    'capture_cleanup': 'capture',
    'knight_advance': 'piece_move',
    'queen_activation': 'piece_move',
    'queen_reposition': 'piece_move',
    'queen_invasion': 'piece_move',
    'queen_blunder': 'piece_move',
    'development': 'piece_move',
    'rook_activation': 'piece_move',
    'castling': 'castling',
    'promotion': 'promotion',
    'promotion_with_check': 'promotion',
    'checkmate': 'checkmate',
    'exchange_chain': 'capture',
    'sacrifice_into_mate': 'capture',
    'passed_pawn_push': 'pawn_move',
    'king_walk': 'piece_move',
    'pawn_spam': 'pawn_move',
    'overextension': 'piece_move',
    'middlegame': 'piece_move',
    'endgame': 'piece_move',
    'rook_hunt': 'piece_move',
  };
  return catMap[cat] || cat;
}

// ── Main ──

async function main() {
  const enginePath = process.argv[2];
  if (!enginePath) {
    console.error('Usage: node run_eval_suite.mjs <path-to-valhalla-engine>');
    process.exit(1);
  }

  // Load samples
  const data = JSON.parse(readFileSync(SAMPLES_PATH, 'utf8'));
  const samples = data.samples;

  // Filter to testable samples (those with moves_to_replay)
  const testable = samples.filter(s => s.moves_to_replay && !s.moves_to_replay.startsWith('n/a'));

  console.log(`\n=== Valhalla Eval Tuning Suite ===`);
  console.log(`Total samples: ${samples.length}`);
  console.log(`Testable (with move replay): ${testable.length}`);
  console.log(`Skipped (need full game replay): ${samples.length - testable.length}\n`);

  // Start engine
  const engine = new Engine(enginePath);
  engine.send('uci');
  await engine.readUntil('uciok');
  engine.send('isready');
  await engine.readUntil('readyok');

  let totalScore = 0;
  let maxScore = 0;
  const results = [];

  for (const sample of testable) {
    maxScore += 3;

    // Set position
    engine.send(`position startpos moves ${sample.moves_to_replay}`);
    engine.send('go depth 4');

    // Read until bestmove
    const lines = await engine.readUntil('bestmove');
    const bestmoveLine = lines.find(l => l.startsWith('bestmove'));
    const infoLines = lines.filter(l => l.startsWith('info'));
    const lastInfo = infoLines[infoLines.length - 1] || '';

    const engineMove = bestmoveLine?.split(' ')[1] || 'none';
    const expectedMove = sample.human_move_freyja; // same notation format
    const isNegative = sample.is_negative || false;

    // Score
    let score = 0;
    let verdict = '';

    if (isNegative) {
      if (engineMove === expectedMove) {
        score = -3;
        verdict = 'ANTI-PATTERN (played the bad move)';
      } else {
        score = 3;
        verdict = 'AVOIDED (correctly avoided bad move)';
      }
    } else {
      if (engineMove === expectedMove) {
        score = 3;
        verdict = 'EXACT MATCH';
      } else {
        const engineCat = categorizeMove(engineMove);
        const expectedCat = categorizeExpected(sample);

        if (engineCat === expectedCat) {
          score = 2;
          verdict = `CATEGORY MATCH (${engineCat})`;
        } else if (engineCat === 'pawn_move' && expectedCat !== 'pawn_move') {
          score = -2;
          verdict = `ANTI-PATTERN (pawn push when human played ${expectedCat})`;
        } else {
          score = 1;
          verdict = `DIFFERENT (engine: ${engineCat}, human: ${expectedCat})`;
        }
      }
    }

    totalScore += score;

    // Extract eval from info line
    const scoreMatch = lastInfo.match(/score\s+cp\s+(-?\d+)/);
    const evalStr = scoreMatch ? `${scoreMatch[1]}cp` : 'n/a';

    results.push({
      id: sample.id,
      name: sample.name,
      engineMove,
      expectedMove,
      score,
      verdict,
      eval: evalStr,
    });

    const scoreStr = score >= 0 ? `+${score}` : `${score}`;
    const icon = score >= 3 ? '++' : score >= 2 ? '+ ' : score >= 1 ? '. ' : score === 0 ? '  ' : '!!';
    console.log(`[${icon}] ${sample.id} ${sample.name.padEnd(35)} engine=${engineMove.padEnd(6)} human=${expectedMove.padEnd(6)} ${scoreStr} ${verdict}`);
  }

  engine.quit();

  // Summary
  console.log(`\n${'='.repeat(80)}`);
  console.log(`TOTAL SCORE: ${totalScore} / ${maxScore} (${Math.round(totalScore/maxScore*100)}%)`);
  console.log(`${'='.repeat(80)}`);

  if (totalScore >= maxScore * 0.67) {
    console.log('VERDICT: PASS — weights are good.');
  } else if (totalScore >= maxScore * 0.50) {
    console.log('VERDICT: MARGINAL — some categories need adjustment.');
  } else {
    console.log('VERDICT: FAIL — fundamental weight problems. DO NOT MERGE.');
  }

  // Category breakdown
  const categories = {};
  for (const r of results) {
    const sample = testable.find(s => s.id === r.id);
    const cat = sample.expected_category;
    if (!categories[cat]) categories[cat] = { total: 0, score: 0, max: 0 };
    categories[cat].total++;
    categories[cat].score += r.score;
    categories[cat].max += 3;
  }

  console.log('\nCategory Breakdown:');
  for (const [cat, data] of Object.entries(categories).sort((a,b) => a[1].score/a[1].max - b[1].score/b[1].max)) {
    const pct = Math.round(data.score / data.max * 100);
    const bar = pct >= 67 ? 'PASS' : pct >= 50 ? 'MARGINAL' : 'FAIL';
    console.log(`  ${cat.padEnd(25)} ${data.score}/${data.max} (${pct}%) ${bar}`);
  }

  console.log('\nPawn Ratio Check:');
  console.log('  Run 10-move self-play and count pawn moves. Target: <= 35% pawn ratio.');
  console.log('  If > 45%, increase development bonus or check that pawn advance has no weight.');

  process.exit(totalScore >= maxScore * 0.50 ? 0 : 1);
}

main().catch(console.error);
