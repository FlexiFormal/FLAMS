import * as vscode from 'vscode';

export class MathHubTreeProvider implements vscode.TreeDataProvider<void> {
  constructor() {}
  getTreeItem(element: void): vscode.TreeItem | Thenable<vscode.TreeItem> {
    return {};
  }
  getChildren(element?: void | undefined): vscode.ProviderResult<void[]> {
    return undefined;
  }
}