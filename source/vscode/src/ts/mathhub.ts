import * as vscode from 'vscode';
import { IMMTContext } from '../extension';

export class MathHubTreeProvider implements vscode.TreeDataProvider<void> {
  constructor(context:IMMTContext) {}
  getTreeItem(element: void): vscode.TreeItem | Thenable<vscode.TreeItem> {
    return {};
  }
  getChildren(element?: void | undefined): vscode.ProviderResult<void[]> {
    return undefined;
  }
}