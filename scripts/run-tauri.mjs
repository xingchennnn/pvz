import { spawn } from 'node:child_process'
import fs from 'node:fs'
import path from 'node:path'

const args = process.argv.slice(2)
const env = { ...process.env }

if (process.platform === 'win32' && env.USERPROFILE) {
  const cargoBin = path.join(env.USERPROFILE, '.cargo', 'bin')
  if (fs.existsSync(cargoBin)) {
    const currentPath = env.Path || env.PATH || ''
    const pathEntries = currentPath.split(path.delimiter).filter(Boolean)
    if (!pathEntries.includes(cargoBin)) {
      env.Path = [cargoBin, currentPath].filter(Boolean).join(path.delimiter)
      env.PATH = env.Path
    }
  }
}

const command = process.platform === 'win32' ? 'tauri' : 'tauri'
const child = spawn(command, args, {
  stdio: 'inherit',
  env,
  shell: process.platform === 'win32',
})

child.on('exit', (code, signal) => {
  if (signal) {
    process.kill(process.pid, signal)
    return
  }
  process.exit(code ?? 1)
})

child.on('error', (error) => {
  console.error(error.message)
  process.exit(1)
})