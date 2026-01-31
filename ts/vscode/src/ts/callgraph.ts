import * as vscode from "vscode";

export class FlamsCallHierarchyTreeProvider implements vscode.TreeDataProvider<CallHierarchyTreeItem> {
  private _onDidChangeTreeData = new vscode.EventEmitter<CallHierarchyTreeItem | undefined>();
  readonly onDidChangeTreeData = this._onDidChangeTreeData.event;

  async getChildren(element?: CallHierarchyTreeItem): Promise<CallHierarchyTreeItem[]> {
    if (!element) {
      // Root level - get call hierarchy for current position
      const editor = vscode.window.activeTextEditor;
      if (!editor) return [];

      const items = await vscode.commands.executeCommand<vscode.CallHierarchyItem[]>(
        'vscode.prepareCallHierarchy',
        editor.document.uri,
        editor.selection.active
      );

      if (!items || items.length === 0) return [];
      return [new CallHierarchyTreeItem(items[0])];
    }

    const calls = await vscode.commands.executeCommand<vscode.CallHierarchyIncomingCall[]>(
        'vscode.provideIncomingCalls',
        element.item
      );
      return calls?.map(call => new CallHierarchyTreeItem(call.from)) || [];
  }

  getTreeItem(element: CallHierarchyTreeItem): vscode.TreeItem {
    return element;
  }

  refresh(): void {
    this._onDidChangeTreeData.fire(undefined);
  }
}

class CallHierarchyTreeItem extends vscode.TreeItem {
  constructor(
    public readonly item: vscode.CallHierarchyItem
  ) {
    super(item.name, vscode.TreeItemCollapsibleState.Collapsed);
    this.description = item.detail;
    this.command = {
      command: 'vscode.open',
      title: 'Open',
      arguments: [item.uri, { selection: item.range }]
    };
  }
}


export class FlamsDocumentSymbolTreeProvider implements vscode.TreeDataProvider<DocumentSymbolTreeItem> {
  private _onDidChangeTreeData = new vscode.EventEmitter<DocumentSymbolTreeItem | undefined>();
  readonly onDidChangeTreeData = this._onDidChangeTreeData.event;

  async getChildren(element?: DocumentSymbolTreeItem): Promise<DocumentSymbolTreeItem[]> {
    if (!element) {
      // Root level - get symbols for active document
      const editor = vscode.window.activeTextEditor;
      if (!editor) {
        return [];
      }

      const symbols = await vscode.commands.executeCommand<vscode.DocumentSymbol[]>(
        'vscode.executeDocumentSymbolProvider',
        editor.document.uri
      );

      if (!symbols || symbols.length === 0) {
        return [];
      }

      return symbols.map(symbol => new DocumentSymbolTreeItem(symbol, editor.document.uri));
    }

    // Return children of this symbol
    if (element.symbol.children && element.symbol.children.length > 0) {
      return element.symbol.children.map(child => 
        new DocumentSymbolTreeItem(child, element.documentUri)
      );
    }

    return [];
  }

  getTreeItem(element: DocumentSymbolTreeItem): vscode.TreeItem {
    return element;
  }

  refresh(): void {
    this._onDidChangeTreeData.fire(undefined);
  }
}

class DocumentSymbolTreeItem extends vscode.TreeItem {
  constructor(
    public readonly symbol: vscode.DocumentSymbol,
    public readonly documentUri: vscode.Uri
  ) {
    super(
      symbol.name,
      symbol.children && symbol.children.length > 0
        ? vscode.TreeItemCollapsibleState.Collapsed
        : vscode.TreeItemCollapsibleState.None
    );

    this.description = symbol.detail;
    this.tooltip = `${vscode.SymbolKind[symbol.kind]}: ${symbol.name}`;
    
    // Set icon based on symbol kind
    this.iconPath = new vscode.ThemeIcon(this.getIconForSymbolKind(symbol.kind));

    // Command to navigate to symbol when clicked
    this.command = {
      command: 'vscode.open',
      title: 'Go to Symbol',
      arguments: [
        documentUri,
        { selection: symbol.selectionRange }
      ]
    };
  }

  private getIconForSymbolKind(kind: vscode.SymbolKind): string {
    // Map symbol kinds to VS Code icons
    const iconMap: { [key: number]: string } = {
      [vscode.SymbolKind.File]: 'file',
      [vscode.SymbolKind.Module]: 'symbol-module',
      [vscode.SymbolKind.Namespace]: 'symbol-namespace',
      [vscode.SymbolKind.Package]: 'symbol-package',
      [vscode.SymbolKind.Class]: 'symbol-class',
      [vscode.SymbolKind.Method]: 'symbol-method',
      [vscode.SymbolKind.Property]: 'symbol-property',
      [vscode.SymbolKind.Field]: 'symbol-field',
      [vscode.SymbolKind.Constructor]: 'symbol-constructor',
      [vscode.SymbolKind.Enum]: 'symbol-enum',
      [vscode.SymbolKind.Interface]: 'symbol-interface',
      [vscode.SymbolKind.Function]: 'symbol-function',
      [vscode.SymbolKind.Variable]: 'symbol-variable',
      [vscode.SymbolKind.Constant]: 'symbol-constant',
      [vscode.SymbolKind.String]: 'symbol-string',
      [vscode.SymbolKind.Number]: 'symbol-number',
      [vscode.SymbolKind.Boolean]: 'symbol-boolean',
      [vscode.SymbolKind.Array]: 'symbol-array',
      [vscode.SymbolKind.Object]: 'symbol-object',
      [vscode.SymbolKind.Key]: 'symbol-key',
      [vscode.SymbolKind.Null]: 'symbol-null',
      [vscode.SymbolKind.EnumMember]: 'symbol-enum-member',
      [vscode.SymbolKind.Struct]: 'symbol-struct',
      [vscode.SymbolKind.Event]: 'symbol-event',
      [vscode.SymbolKind.Operator]: 'symbol-operator',
      [vscode.SymbolKind.TypeParameter]: 'symbol-type-parameter',
    };

    return iconMap[kind] || 'symbol-misc';
  }
}
