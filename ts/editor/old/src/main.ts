import './style.css'
import './workers'
console.log("Here: main.ts")


import * as editor from './editor'
//let editor = await import('./editor')

import * as setup from './setup'
//let setup = await import('./setup')

//import './latex/extension'
//import './stex'
async function extensions() {
  console.log("Here: initMonaco start")
  await setup.initMonaco()
  console.log("Here: initMonaco done")
  import('./latex/extension')
  import('./stex')
}

interface CodeEditor {
  dispose():void
}

//import * as monaco from 'monaco-editor'
//const monaco = await import('monaco-editor')
let simple: CodeEditor | undefined = undefined

const snippet = `\\documentclass{article}
\\usepackage{stex}
\\usemodule[sTeX/Algebra/General]{mod?Group}
\\begin{document}
\\vardef{vG}[name=G]{G}

Let $\\vG$ a \\sn{group}
\\end{document}
`

export async function henlofoo() {
  extensions()
  await editor.bindSimpleEditor('myeditor','/tmp/test.tex',snippet)
}

document.querySelector('#toggleSimple')!.addEventListener('click', async () => {
  if (simple) {
    simple.dispose()
    simple = undefined
    return
  }
  await henlofoo()
})

document.querySelector('#toggleFull')!.addEventListener('click', async () => {
  extensions()
  return editor.bindFullEditor('#editors')
})

document.querySelector('#open1')!.addEventListener('click', async () => {
  editor.openTab('/tmp/test.tex',snippet)
})

document.querySelector('#open2')!.addEventListener('click', async () => {
  editor.openTab('/tmp/narf.txt','henlo wrld')
})