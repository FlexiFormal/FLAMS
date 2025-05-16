import { ExtensionHostKind, IExtensionManifest, registerExtension } from 'vscode/extensions'

console.log("Here: extension.ts")

let manifest = {
  "name": "latex-workshop",
  "displayName": "LaTeX Workshop",
  "engines": {
    "vscode": "*"
  },
  "version": "9.14.1",
  "publisher": "James-Yu",
  "activationEvents": ["*"],
  "contributes": {
    "languages": [
      {
        "id": "tex",
        "aliases": [
          "TeX",
          "tex"
        ],
        "extensions": [
          ".sty",
          ".cls",
          ".bbx",
          ".cbx"
        ],
        "configuration": "./syntax/latex-language-configuration.json"
      },
      {
        "id": "doctex",
        "aliases": [
          "DocTeX",
          "doctex"
        ],
        "extensions": [
          ".dtx"
        ],
        "configuration": "./syntax/doctex-language-configuration.json"
      },
      {
        "id": "latex",
        "aliases": [
          "LaTeX",
          "latex"
        ],
        "extensions": [
          ".tex",
          ".ltx",
          ".ctx"
        ],
        "configuration": "./syntax/latex-language-configuration.json"
      },
      {
        "id": "bibtex",
        "aliases": [
          "BibTeX",
          "bibtex"
        ],
        "extensions": [
          ".bib"
        ],
        "configuration": "./syntax/bibtex-language-configuration.json"
      },
      {
        "id": "bibtex-style",
        "aliases": [
          "BibTeX style"
        ],
        "extensions": [
          ".bst"
        ],
        "configuration": "./syntax/bibtex-style-language-configuration.json"
      },
      {
        "id": "latex-expl3",
        "aliases": [
          "LaTeX-Expl3"
        ],
        "configuration": "./syntax/latex3-language-configuration.json"
      },
      {
        "id": "markdown_latex_combined",
        "configuration": "./syntax/markdown-latex-combined-language-configuration.json"
      }
    ],
    "grammars": [
      {
        "language": "tex",
        "scopeName": "text.tex",
        "path": "./syntax/TeX.tmLanguage.json"
      },
      {
        "language": "doctex",
        "scopeName": "text.tex.doctex",
        "path": "./syntax/DocTeX.tmLanguage.json"
      },
      {
        "language": "latex",
        "scopeName": "text.tex.latex",
        "path": "./syntax/LaTeX.tmLanguage.json",
        "embeddedLanguages": {
          "source.asymptote": "asymptote",
          "source.cpp": "cpp_embedded_latex",
          "source.css": "css",
          "source.dot": "dot",
          "source.gnuplot": "gnuplot",
          "text.html": "html",
          "source.java": "java",
          "source.js": "javascript",
          "source.julia": "julia",
          "source.lua": "lua",
          "source.python": "python",
          "source.ruby": "ruby",
          "source.scala": "scala",
          "source.ts": "typescript",
          "text.xml": "xml",
          "source.yaml": "yaml",
          "meta.embedded.markdown_latex_combined": "markdown_latex_combined"
        }
      },
      {
        "language": "bibtex",
        "scopeName": "text.bibtex",
        "path": "./syntax/Bibtex.tmLanguage.json"
      },
      {
        "language": "bibtex-style",
        "scopeName": "source.bst",
        "path": "./syntax/BibTeX-style.tmLanguage.json"
      },
      {
        "language": "latex-expl3",
        "scopeName": "text.tex.latex.expl3",
        "path": "./syntax/LaTeX-Expl3.tmLanguage.json"
      },
      {
        "language": "markdown_latex_combined",
        "scopeName": "text.tex.markdown_latex_combined",
        "path": "./syntax/markdown-latex-combined.tmLanguage.json"
      }
    ]
  }
}


const { registerFileUrl: registerFile } = await registerExtension(manifest as IExtensionManifest, ExtensionHostKind.LocalWebWorker)

const files = [
  "latex-language-configuration.json",
  "doctex-language-configuration.json",
  "bibtex-language-configuration.json",
  "bibtex-style-language-configuration.json",
  "latex3-language-configuration.json",
  "markdown-latex-combined-language-configuration.json",
  "TeX.tmLanguage.json",
  "DocTeX.tmLanguage.json",
  "LaTeX.tmLanguage.json",
  "Bibtex.tmLanguage.json",
  "BibTeX-style.tmLanguage.json",
  "LaTeX-Expl3.tmLanguage.json",
  "markdown-latex-combined.tmLanguage.json"
]
files.forEach(file => registerFile(`./syntax/${file}`, `syntax/${file}`))