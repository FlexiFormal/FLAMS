import { LogLevel } from 'vscode/services';
import getKeybindingsServiceOverride from '@codingame/monaco-vscode-keybindings-service-override';
import { MonacoEditorLanguageClientWrapper, WrapperConfig } from 'monaco-editor-wrapper';
import { configureMonacoWorkers } from './utils/utils';
import * as vscode from 'vscode';
import { getWS } from './utils/websockets';
import { latexExtension } from './languages/latex';

import { RegisteredFileSystemProvider, registerFileSystemOverlay, RegisteredMemoryFile } from '@codingame/monaco-vscode-files-service-override';



export function mountEditor(element:HTMLElement,lsp:string): MonacoEditorLanguageClientWrapper {
  const connection = getWS(lsp);

  /*
  const file1 = vscode.Uri.file("/workspace/test.tex");
  const file2 = vscode.Uri.file("/workspace/test2.tex");

  const fileSystemProvider = new RegisteredFileSystemProvider(false);
  fileSystemProvider.registerFile(new RegisteredMemoryFile(file1, text1));
  fileSystemProvider.registerFile(new RegisteredMemoryFile(file2, text2));
  registerFileSystemOverlay(1, fileSystemProvider);
*/

  const config = <WrapperConfig>{
    logLevel: LogLevel.Debug,
    vscodeApiConfig: {
        userServices: {
            ...getKeybindingsServiceOverride()
        },
        userConfiguration: {
            json: JSON.stringify({
                'workbench.colorTheme': 'Default Dark Modern',
                'editor.guides.bracketPairsHorizontal': 'active',
                'editor.lightbulb.enabled': 'On',
                'editor.wordBasedSuggestions': 'off',
                'editor.experimental.asyncTokenization': true
            })
        }
    },
    editorAppConfig: {
        $type: 'extended',
        codeResources: {
            main: {
                text:text1,
                fileExt: 'tex',
                //uri:file1.fsPath,
                enforceLanguageId: "latex"
            }
        },
        extensions:[latexExtension],
        useDiffEditor: false,
        monacoWorkerFactory: configureMonacoWorkers,
        htmlContainer: element
    },
    languageClientConfigs: {
        stex: {
            languageId: 'stex',
            connection: connection,
            clientOptions: {
                documentSelector: ['tex'],
                workspaceFolder: {
                    index: 0,
                    name: 'workspace',
                    uri: vscode.Uri.parse("/workspace")
                },
            }
        }
    }
  };
  const wrapper = new MonacoEditorLanguageClientWrapper();

  wrapper.init(config).then(async () => {
    /*console.log(`opening ${file1.fsPath}`);
    await vscode.workspace.openTextDocument(file1);
    console.log(`opening ${file2.fsPath}`);
    await vscode.workspace.openTextDocument(file2);
    console.log(`starting`);*/
    await wrapper.start().then(() => {
        console.log("started!");
    });
  });
  return wrapper;
}

/*async function test() {

  const config = mountEditor(document.getElementById('monaco-editor-root')!);
  await wrapper.initAndStart(config);
  // await wrapper.dispose();
};


test();
*/
const text1 = `\\documentclass{article}
\\usepackage{stex}
\\usemodule[sTeX/Algebra/General]{mod?Group}
\\begin{document}
\\vardef{vG}[name=G]{G}

Let $\\vG$ a \\sn{group}
\\end{document}
`;

const text2 = `\\documentclass{article}
\\usepackage{stex}
\\usemodule[sTeX/Logic/General]{mod?Language}
\\begin{document}
\\vardef{vG}[name=G]{G}

Let $\\vG$ a \\sn{language}
\\end{document}
`;