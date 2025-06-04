import { IResolvedTextEditorModel, IReference, OpenEditor, attachPart } from '@codingame/monaco-vscode-views-service-override'
import * as monaco from 'monaco-editor'
import { createConfiguredEditor } from 'vscode/monaco'

console.log("Here: editor.ts")

let currentEditor: ({
  modelRef: IReference<IResolvedTextEditorModel>
  editor: monaco.editor.IStandaloneCodeEditor
} & monaco.IDisposable) | null = null


import {
  Parts,
  onPartVisibilityChange,
  isPartVisibile,
} from '@codingame/monaco-vscode-views-service-override'

export async function bindFullEditor(elem:string) {
  const el = document.querySelector<HTMLDivElement>(elem)!
  const part = Parts.EDITOR_PART
  attachPart(part, el)
  if (!isPartVisibile(part)) {
    el.style.display = 'none'
  }
  onPartVisibilityChange(part, visible => {
    el.style.display = visible ? 'block' : 'none'
  })
}


import * as vscode from 'vscode'
export async function openTab(file:string,contents:string) {
  let modelRef = await createModelReference(monaco.Uri.file(file),contents)
  let doc = await vscode.workspace.openTextDocument(modelRef.object.textEditorModel!.uri)
  await vscode.window.showTextDocument(doc, {preview: false})
}


import { createModelReference } from 'vscode/monaco'

export async function bindSimpleEditor(elem:string,file:string,contents:string):Promise<monaco.editor.IStandaloneCodeEditor> {
  let modelRef = await createModelReference(monaco.Uri.file(file),contents)
  return createConfiguredEditor(document.getElementById(elem)!, {
    model: modelRef.object.textEditorModel,
    automaticLayout: true
  })
}

export const openNewCodeEditor: OpenEditor = async (modelRef) => {
  if (currentEditor != null) {
    currentEditor.dispose()
    currentEditor = null
  }
  const container = document.createElement('div')
  container.style.position = 'fixed'
  container.style.backgroundColor = 'rgba(0, 0, 0, 0.5)'
  container.style.top = container.style.bottom = container.style.left = container.style.right = '0'
  container.style.cursor = 'pointer'

  const editorElem = document.createElement('div')
  editorElem.style.position = 'absolute'
  editorElem.style.top = editorElem.style.bottom = editorElem.style.left = editorElem.style.right = '0'
  editorElem.style.margin = 'auto'
  editorElem.style.width = '80%'
  editorElem.style.height = '80%'

  container.appendChild(editorElem)

  document.body.appendChild(container)
  try {
    const editor = createConfiguredEditor(
      editorElem,
      {
        model: modelRef.object.textEditorModel,
        readOnly: true,
        automaticLayout: true
      }
    )

    currentEditor = {
      dispose: () => {
        editor.dispose()
        modelRef.dispose()
        document.body.removeChild(container)
        currentEditor = null
      },
      modelRef,
      editor
    }

    editor.onDidBlurEditorWidget(() => {
      currentEditor?.dispose()
    })
    container.addEventListener('mousedown', (event) => {
      if (event.target !== container) {
        return
      }

      currentEditor?.dispose()
    })

    return editor
  } catch (error) {
    document.body.removeChild(container)
    currentEditor = null
    throw error
  }
}
