import { ExtensionConfig } from "monaco-editor-wrapper"

const manifest = {
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

import latexConf from './syntax/latex-language-configuration.json?raw';
import doctexConf from './syntax/doctex-language-configuration.json?raw';
import bibtexConf from './syntax/bibtex-language-configuration.json?raw';
import bibtexStyleConf from './syntax/bibtex-style-language-configuration.json?raw';
import latex3Conf from './syntax/latex3-language-configuration.json?raw';
import mdlatexConf from './syntax/markdown-latex-combined-language-configuration.json?raw';
import texTm from './syntax/TeX.tmLanguage.json?raw';
import doctexTm from './syntax/DocTeX.tmLanguage.json?raw';
import latexTm from './syntax/LaTeX.tmLanguage.json?raw';
import bibtexTm from './syntax/Bibtex.tmLanguage.json?raw';
import bibtexStyleTm from './syntax/BibTeX-style.tmLanguage.json?raw';
import explTm from './syntax/LaTeX-Expl3.tmLanguage.json?raw';
import mdltTm from './syntax/markdown-latex-combined.tmLanguage.json?raw';

const fileMap = new Map<string, string | URL>();
fileMap.set('/syntax/latex-language-configuration.json',latexConf);
fileMap.set('/syntax/doctex-language-configuration.json',doctexConf);
fileMap.set('/syntax/bibtex-language-configuration.json',bibtexConf);
fileMap.set('/syntax/bibtex-style-language-configuration.json',bibtexStyleConf);
fileMap.set('/syntax/latex3-language-configuration.json',latex3Conf);
fileMap.set('/syntax/markdown-latex-combined-language-configuration.json',mdlatexConf);

fileMap.set('/syntax/TeX.tmLanguage.json',texTm);
fileMap.set('/syntax/DocTeX.tmLanguage.json',doctexTm);
fileMap.set('/syntax/LaTeX.tmLanguage.json',latexTm);
fileMap.set('/syntax/Bibtex.tmLanguage.json',bibtexTm);
fileMap.set('/syntax/BibTeX-style.tmLanguage.json',bibtexStyleTm);
fileMap.set('/syntax/LaTeX-Expl3.tmLanguage.json',explTm);
fileMap.set('/syntax/markdown-latex-combined.tmLanguage.json',unescape(encodeURIComponent(mdltTm)));

export const latexExtension = <ExtensionConfig>{
  config:manifest,
  filesOrContents:fileMap
};