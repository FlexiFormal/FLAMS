import { ILogService, LogLevel, StandaloneServices, initialize as initializeMonacoService } from 'vscode/services'
import { initialize as initializeVscodeExtensions } from 'vscode/extensions'
import getModelServiceOverride from '@codingame/monaco-vscode-model-service-override'
import getNotificationServiceOverride from '@codingame/monaco-vscode-notifications-service-override'
import getDialogsServiceOverride from '@codingame/monaco-vscode-dialogs-service-override'
import getConfigurationServiceOverride from '@codingame/monaco-vscode-configuration-service-override'
import getKeybindingsServiceOverride from '@codingame/monaco-vscode-keybindings-service-override'
import getTextmateServiceOverride from '@codingame/monaco-vscode-textmate-service-override'
import getThemeServiceOverride from '@codingame/monaco-vscode-theme-service-override'
import getLanguagesServiceOverride from '@codingame/monaco-vscode-languages-service-override'
import getViewsServiceOverride, {isEditorPartVisible} from '@codingame/monaco-vscode-views-service-override'
import getBannerServiceOverride from '@codingame/monaco-vscode-view-banner-service-override'
import getStatusBarServiceOverride from '@codingame/monaco-vscode-view-status-bar-service-override'
import getTitleBarServiceOverride from '@codingame/monaco-vscode-view-title-bar-service-override'
import getPreferencesServiceOverride from '@codingame/monaco-vscode-preferences-service-override'
import getQuickAccessServiceOverride from '@codingame/monaco-vscode-quickaccess-service-override'
import getOutputServiceOverride from '@codingame/monaco-vscode-output-service-override'
import getAccessibilityServiceOverride from '@codingame/monaco-vscode-accessibility-service-override'
import getExtensionServiceOverride from '@codingame/monaco-vscode-extensions-service-override'
import getEnvironmentServiceOverride from '@codingame/monaco-vscode-environment-service-override'
import * as monaco from 'monaco-editor'
import { openNewCodeEditor } from './editor'
import { getExtensionHostWorker,initWorkers } from './workers'

import '@codingame/monaco-vscode-theme-defaults-default-extension'


import { createModelReference } from 'vscode/monaco'

console.log("Here: setup.ts")


console.log("Here: setup/initMonaco/initializeMonacoService")
// Override services
await initializeMonacoService({
  ...getExtensionServiceOverride(getExtensionHostWorker()),
  ...getModelServiceOverride(),
  ...getNotificationServiceOverride(),
  ...getDialogsServiceOverride(),
  ...getConfigurationServiceOverride(monaco.Uri.file('/tmp')),
  ...getKeybindingsServiceOverride(),
  ...getTextmateServiceOverride(),
  ...getThemeServiceOverride(),
  ...getLanguagesServiceOverride(),
  ...getPreferencesServiceOverride(),
  ...getViewsServiceOverride(openNewCodeEditor),
  ...getBannerServiceOverride(),
  ...getStatusBarServiceOverride(),
  ...getTitleBarServiceOverride(),
  ...getQuickAccessServiceOverride({
    isKeybindingConfigurationVisible: isEditorPartVisible,
    shouldUseGlobalPicker: (_editor, isStandalone) => !isStandalone && isEditorPartVisible()
  }),
  ...getOutputServiceOverride(),
  ...getAccessibilityServiceOverride(),
  ...getEnvironmentServiceOverride({
    remoteAuthority:undefined,
    enableWorkspaceTrust: false
  })
})

export async function initMonaco() {
  console.log("Here: setup/initMonaco")
  await initWorkers()
  StandaloneServices.get(ILogService).setLevel(LogLevel.Off)
  console.log("Here: setup/initMonaco/initializeVscodeExtensions")
  await initializeVscodeExtensions()

  await createModelReference(monaco.Uri.from({ scheme: 'user', path: '/settings.json' }), `{
    "workbench.colorTheme": "Default Dark+",
    "workbench.iconTheme": "vs-seti",
    "editor.autoClosingBrackets": "languageDefined",
    "editor.autoClosingQuotes": "languageDefined",
    "editor.scrollBeyondLastLine": false,
    "editor.mouseWheelZoom": true,
    "editor.wordBasedSuggestions": false,
    "editor.acceptSuggestionOnEnter": "on",
    "editor.foldingHighlight": true,
    "editor.semanticHighlighting.enabled": true,
    "editor.bracketPairColorization.enabled": true,
    "editor.fontSize": 12,
    "audioCues.lineHasError": "on",
    "audioCues.onDebugBreak": "on",
    "debug.toolBarLocation": "docked",
    "editor.experimental.asyncTokenization": true,
    "terminal.integrated.tabs.title": "\${sequence}",
    "typescript.tsserver.log": "normal"
  }`)
  console.log("Here: setup/initMonaco done")
}

//await initMonaco()
