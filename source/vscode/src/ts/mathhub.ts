import * as vscode from 'vscode';
import { get_context, IMMTContext } from '../extension';
import * as immt from './immt/types';
import { IMMTServer } from './immt/server';
import path from 'path';
import * as fs from 'fs';

export class MathHubTreeProvider implements vscode.TreeDataProvider<AnyMH> {
  primary_server:IMMTServer;
  remote_server:IMMTServer | undefined;
  mathhubs:string[] | undefined;
  constructor(context:IMMTContext) {
    this.primary_server = context.server;
    this.remote_server = context.remote_server;
  }
  getTreeItem(element: AnyMH): vscode.TreeItem | Thenable<vscode.TreeItem> {
    return element;
  }
  async getChildren(element?: AnyMH | undefined): Promise<AnyMH[]> {
    if (!this.mathhubs) {
      const mhs = await this.primary_server.api_settings();
      if (!mhs) {
        this.mathhubs = [];
      } else {
        this.mathhubs = mhs.mathhubs;
      }
    }
    if (!element) {
      const ret = await this.ga_from_server();
      if (ret) {
        const [group_entries,archive_entries] = ret;
        const entries = (<AnyMH[]>group_entries).concat(archive_entries);
        return entries;
      } else {
        return [];
      }
    }
    if (element instanceof ArchiveGroup) {
      const ret = await this.ga_from_server(element);
      if (ret) {
        const [group_entries,archive_entries] = ret;
        const entries = (<AnyMH[]>group_entries).concat(archive_entries);
        return entries;
      } else {
        return [];
      }
    }
    if (element instanceof Archive) {
      const ret = await this.df_from_server(element);
      if (ret) {
        const [dirs,files] = ret;
        const entries = (<AnyMH[]>dirs).concat(files);
        return entries;
      }
    }
    if (element instanceof Dir) {
      const ret = await this.df_from_server(element.archive,element.rel_path);
      if (ret) {
        const [dirs,files] = ret;
        const entries = (<AnyMH[]>dirs).concat(files);
        return entries;
      }
    }
    return [];
  }

  private async df_from_server(a:Archive,rp?:string): Promise<[Dir[],File[]] | undefined> {
    const entries = await (a.local? this.primary_server.backend_archive_entries(a.id,rp) : this.remote_server?.backend_archive_entries(a.id,rp));
    if (!entries) {
      vscode.window.showErrorMessage("iMMT: No file entries found");
      return;
    }
    const [dirs,files] = entries;
    const dirs_e = dirs.map(d => new Dir(a,d));
    const files_e = files.map(f => new File(a,f));
    return [dirs_e,files_e];
  }

  private async ga_from_server(in_group?:ArchiveGroup): Promise<[ArchiveGroup[],Archive[]] | undefined> {
    if (!in_group || in_group.lr === LRB.Both) {
      return await this.ga_from_both_servers(in_group?.id);
    }
    if (in_group.lr === LRB.Local) {
      return await this.ga_from_local_server(in_group.id);
    }
    return await this.ga_from_remote_server(in_group.id);
  }

  private async ga_from_local_server(id:string): Promise<[ArchiveGroup[],Archive[]] | undefined> {
    const entries = await this.primary_server.backend_group_entries(id);
    if (!entries) {
      vscode.window.showErrorMessage("iMMT: No archives found");
      return;
    }
    const [groups,archives] = entries;
    const group_entries = groups.map(g => new ArchiveGroup(g,LRB.Local));
    const archive_entries = archives.map(a => new Archive(a,true,this.mathhubs));
    return [group_entries,archive_entries];
  }

  private async ga_from_remote_server(id:string): Promise<[ArchiveGroup[],Archive[]] | undefined> {
    if (!this.remote_server) {
      vscode.window.showErrorMessage(`iMMT: No remote server set`);
      return;
    }
    const entries = await this.remote_server.backend_group_entries(id);
    if (!entries) {
      vscode.window.showErrorMessage("iMMT: No archives found");
      return;
    }
    const [groups,archives] = entries;
    const group_entries = groups.map(g => new ArchiveGroup(g,LRB.Remote));
    const archive_entries = archives.map(a => new Archive(a,false));
    return [group_entries,archive_entries];
  }

  private async ga_from_both_servers(id?:string): Promise<[ArchiveGroup[],Archive[]] | undefined> {
    const entries = await this.primary_server.backend_group_entries(id);
    if (!entries) {
      vscode.window.showErrorMessage("iMMT: No archives found");
      return;
    }
    const [groups,archives] = entries;
    const group_entries = groups.map(g => new ArchiveGroup(g,LRB.Local));
    const archive_entries = archives.map(a => new Archive(a,true,this.mathhubs));
    if (this.remote_server) {
      const remote_entries = await this.remote_server.backend_group_entries(id);
      if (!remote_entries) {
        vscode.window.showErrorMessage(`iMMT: Failed to query remote server at ${this.remote_server.url}`);
        return [group_entries,archive_entries];
      }
      const [remote_groups,remote_archives] = remote_entries;
      remote_groups.forEach(g => {
        const old = group_entries.find(o => o.id === g.id);
        if (old) {
          old.lr = LRB.Both;
        } else {
          group_entries.push(new ArchiveGroup(g,LRB.Remote));
        }
      });
      remote_archives.forEach(a => {
        const old = archive_entries.find(o => o.id === a.id);
        if (!old) {
          archive_entries.push(new Archive(a,false));
        }
      });
    }
    return [group_entries,archive_entries];
  }
}

type AnyMH = ArchiveGroup | Archive | Dir | File;

enum LRB {
  Local = 0,
  Remote = 1,
  Both = 2
}

class ArchiveGroup extends vscode.TreeItem {
  id:string;
  lr:LRB;
  constructor(group:immt.ArchiveGroup,lr:LRB) {
    const name = group.id.split("/").pop();
    if (!name) {
      throw new Error("iMMT: Invalid archive group name");
    }
    super(name,vscode.TreeItemCollapsibleState.Collapsed);
    this.id = group.id;
    this.lr = lr;
    this.iconPath = (lr === LRB.Local || lr === LRB.Both) ? 
      new vscode.ThemeIcon("library") : 
      vscode.Uri.joinPath(get_context().vsc.extensionUri,"img","MathHub.svg")
    ;
  }
}
class Archive extends vscode.TreeItem {
  id:string;
  local:boolean;
  constructor(archive:immt.Archive,local:boolean,mhs?:string[]) {
    const name = archive.id.split("/").pop();
    if (!name) {
      throw new Error("iMMT: Invalid archive name");
    }
    super(name,vscode.TreeItemCollapsibleState.Collapsed);
    this.id = archive.id;
    this.local = local;
    this.iconPath = local ? 
      new vscode.ThemeIcon("book") : 
      vscode.Uri.joinPath(get_context().vsc.extensionUri,"img","MathHub.svg")
    ;
    if (local && mhs) {
      mhs.find(mh => {
        const fp = archive.id.split("/").reduce((p,seg) => path.join(p,seg),mh);
        if (fs.existsSync(fp)) {
          this.resourceUri = vscode.Uri.file(fp);
          this.tooltip = fp;
          return true;
        } else {return false;}
      });
    }
  }
}

class Dir extends vscode.TreeItem {
  archive:Archive;
  rel_path:string;

  constructor(archive:Archive,dir:immt.Directory) {
    const name = dir.rel_path.split("/").pop();
    if (!name) {
      throw new Error("iMMT: Invalid directory name");
    }
    super(name,vscode.TreeItemCollapsibleState.Collapsed);
    this.id = `[${archive.id}]${dir.rel_path}`;
    this.archive = archive;
    this.rel_path = dir.rel_path;
    this.iconPath = new vscode.ThemeIcon("file-directory");
    if (archive.resourceUri) {
      this.resourceUri = vscode.Uri.file(
        this.rel_path.split("/").reduce(
          (p,seg) => path.join(p,seg),
          path.join(archive.resourceUri.fsPath,"source")
        )
      );
    }
  }

}

class File extends vscode.TreeItem {
  archive:Archive;
  rel_path:string;
  constructor(archive:Archive,file:immt.File) {
    const name = path.basename(file.rel_path);
    super(name,vscode.TreeItemCollapsibleState.None);
    this.archive = archive;
    this.id = `[${archive.id}]${file.rel_path}`;
    this.rel_path = file.rel_path;
    this.iconPath = new vscode.ThemeIcon("file");
    if (archive.resourceUri) {
      this.resourceUri = vscode.Uri.file(
        this.rel_path.split("/").reduce(
          (p,seg) => path.join(p,seg),
          path.join(archive.resourceUri.fsPath,"source")
        )
      );
    }
  }
}