import * as vscode from 'vscode';
import { getContext, FLAMSContext, awaitContext } from '../extension';
import * as FLAMS from '@kwarc/flams';
import path from 'path';
import * as fs from 'fs';
import { Commands } from './commands';

type FLAMSServer = FLAMS.FLAMSServer;

export interface Settings {
  mathhubs: string[],
  debug:boolean,
  server: {
    port:number,
    ip:string
    database?:string
  },
  log_dir:string,
  buildqueue: {
    num_threads:number
  }
}

let settings : Settings | undefined;
export async function getSettings() : Promise<Settings | undefined> {
  if (!settings) {
    const context = await awaitContext();
    settings = await apiSettings(context.server);
  }
  return settings;
} 

interface InstallParams {
  archives:string[],
  remote_url:string
}

async function apiSettings(server:FLAMSServer): Promise<Settings | undefined> {
  const ret = await server.rawPostRequest<{},[Settings,any,any] | undefined>("api/settings",{});
  if (ret) {
    const [settings,_] = ret;
    return settings;
  }
}

export class MathHubTreeProvider implements vscode.TreeDataProvider<AnyMH> {
  primary_server:FLAMSServer;
  remote_server:FLAMSServer | undefined;
  mathhubs:string[] | undefined;
  roots:AnyMH[] | undefined;

  constructor(context:FLAMSContext) {
    this.primary_server = context.server;
    this.remote_server = context.remote_server;
  }



  onUpdated = new vscode.EventEmitter<void>();
  onDidChangeTreeDataI = new vscode.EventEmitter<AnyMH | undefined | null | void>();
  onDidChangeTreeData?: vscode.Event<void | AnyMH | AnyMH[] | null | undefined> | undefined =
      this.onDidChangeTreeDataI.event;

  async install(item:Archive|ArchiveGroup):Promise<void> {
    if (!this) {
      throw new Error("MathHubTreeProvider not initialized");
    }
    const nodeps_downloads = (item instanceof Archive)? [item.id] : filter_things(item);
    vscode.window.withProgress({ location: { viewId: "flams-mathhub" } }, () => new Promise((r) => this.onUpdated.event(r)));
    const context = getContext();
    if (!context.remote_server) {
      return;
    }
    const downloads = await context.remote_server.archiveDependencies(nodeps_downloads);
    if (!downloads) { return; }

    const remote_url = context.remote_server.url;
    getContext().client.sendNotification("flams/install",<InstallParams>{archives:nodeps_downloads.concat(downloads),remote_url:remote_url});;
    /*this.roots?.forEach((mhti: AnyMH) => {
      mhti.contextValue = "disabled";
    });*/
    this.onDidChangeTreeDataI.fire();
  }

  async update() {
    this.roots = undefined;
    this.onDidChangeTreeDataI.fire();
    this.onUpdated.fire();
  }


  getTreeItem(element: AnyMH): vscode.TreeItem | Thenable<vscode.TreeItem> {
    return element;
  }
  async getChildren(element?: AnyMH | undefined): Promise<AnyMH[]> {
    if (!this.mathhubs) {
      const mhs = await getSettings();
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
        this.roots = entries;
        return entries;
      } else {
        this.roots = [];
        return [];
      }
    }
    if (element instanceof ArchiveGroup) {
      return element.children;
      /*
      const ret = await this.ga_from_server(element);
      if (ret) {
        const [group_entries,archive_entries] = ret;
        const entries = (<AnyMH[]>group_entries).concat(archive_entries);
        return entries;
      } else {
        return [];
      }
        */
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
    const entries = await (a.local? this.primary_server.backendArchiveEntries(a.id,rp) : this.remote_server?.backendArchiveEntries(a.id,rp));
    if (!entries) {
      vscode.window.showErrorMessage("𝖥𝖫∀𝖬∫: No file entries found");
      return;
    }
    const [dirs,files] = entries;
    const dirs_e = dirs.map(d => new Dir(a,d));
    const files_e = files.map(f => new File(a,f));
    return [dirs_e,files_e];
  }

  private async ga_from_server(in_group?:ArchiveGroup): Promise<[ArchiveGroup[],Archive[]] | undefined> {
    if (!in_group || in_group.lr === LRB.Both) {
      return await this.ga_from_both_servers(in_group);
    }
    if (in_group.lr === LRB.Local) {
      return await this.ga_from_local_server(in_group);
    }
    return await this.ga_from_remote_server(in_group);
  }

  private async ga_from_local_server(in_group:ArchiveGroup): Promise<[ArchiveGroup[],Archive[]] | undefined> {
    const entries = await this.primary_server.backendGroupEntries(in_group.id);
    if (!entries) {
      vscode.window.showErrorMessage("𝖥𝖫∀𝖬∫: No archives found");
      return;
    }
    const [groups,archives] = entries;
    const igroup_entries = groups.map(async g => {
      const ng = new ArchiveGroup(g,LRB.Local,in_group);
      const children = await this.ga_from_local_server(ng);
      if (children) {
        const [group_entries,archive_entries] = children;
        ng.children = (<(ArchiveGroup|Archive)[]>group_entries).concat(archive_entries);
      }
      return ng;
    });
    const group_entries= await Promise.all(igroup_entries);
    const archive_entries = archives.map(a => new Archive(a,true,false,in_group,this.mathhubs));
    return [group_entries,archive_entries];
  }

  private async ga_from_remote_server(in_group:ArchiveGroup): Promise<[ArchiveGroup[],Archive[]] | undefined> {
    if (!this.remote_server) {
      vscode.window.showErrorMessage(`𝖥𝖫∀𝖬∫: No remote server set`);
      return;
    }
    const entries = await this.remote_server.backendGroupEntries(in_group.id).catch(() => undefined);
    if (!entries) {
      vscode.window.showErrorMessage(`𝖥𝖫∀𝖬∫: No archives found in ${in_group.id}`);
      return;
    }
    const [groups,archives] = entries;
    const igroup_entries = groups.map(async g => {
      const ng = new ArchiveGroup(g,LRB.Remote,in_group);
      const children = await this.ga_from_remote_server(ng);
      if (children) {
        const [group_entries,archive_entries] = children;
        ng.children = (<(ArchiveGroup|Archive)[]>group_entries).concat(archive_entries);
      }
      return ng;
    });
    const group_entries= await Promise.all(igroup_entries);
    const archive_entries = archives.map(a => new Archive(a,false,a.git?true:false,in_group));
    return [group_entries,archive_entries];
  }

  private async ga_from_both_servers(in_group?:ArchiveGroup): Promise<[ArchiveGroup[],Archive[]] | undefined> {
    const entries = await this.primary_server.backendGroupEntries(in_group?.id);
    if (!entries) {
      if (in_group) {
        vscode.window.showErrorMessage(`𝖥𝖫∀𝖬∫: No archives found in ${in_group.id}`);
      } else {
        vscode.window.showErrorMessage("𝖥𝖫∀𝖬∫: No archives found");
      }
      return;
    }
    const [groups,archives] = entries;
    const igroup_entries = groups.map(async g => {
      const ng = new ArchiveGroup(g,LRB.Local);
      const children = await this.ga_from_local_server(ng);
      if (children) {
        const [group_entries,archive_entries] = children;
        ng.children = (<(ArchiveGroup|Archive)[]>group_entries).concat(archive_entries);
      }
      return ng;
    });
    const group_entries= await Promise.all(igroup_entries);
    const archive_entries = archives.map(a => new Archive(a,true,false,in_group,this.mathhubs));
    if (this.remote_server) {
      const remote_entries = await this.remote_server.backendGroupEntries(in_group?.id).catch(() => undefined);
      if (!remote_entries) {
        vscode.window.showErrorMessage(`𝖥𝖫∀𝖬∫: Failed to query remote server at ${this.remote_server.url}`);
        return [group_entries,archive_entries];
      }
      const [remote_groups,remote_archives] = remote_entries;
      await Promise.all(remote_groups.map(async g => {
        const old = group_entries.find(o => o.id === g.id);
        if (old) {
          old.lr = LRB.Both;
          const children = await this.ga_from_remote_server(old);
          if (children) {
            const [group_entries,archive_entries] = children;
            merge(old.children,(<(ArchiveGroup|Archive)[]>group_entries).concat(archive_entries));
          }
          old.update();
        } else {
          const ng = new ArchiveGroup(g,LRB.Remote,in_group);
          const children = await this.ga_from_remote_server(ng);
          if (children) {
            const [group_entries,archive_entries] = children;
            ng.children = (<(ArchiveGroup|Archive)[]>group_entries).concat(archive_entries);
          }
          group_entries.push(ng);
        }
      }));
      remote_archives.forEach(a => {
        const old = archive_entries.find(o => o.id === a.id);
        if (!old) {
          archive_entries.push(new Archive(a,false,a.git?true:false,in_group));
        }
      });
    }
    return [group_entries,archive_entries];
  }
}

function filter_things(item:ArchiveGroup): string[] {
  var ret: string[] = [];
  for (const child of item.children) {
    if (child instanceof Archive && child.downloadable) {
      ret.push(child.id);
    } else if (child instanceof ArchiveGroup) {
      ret = ret.concat(filter_things(child));
    }
  }
  return ret;
}

type AnyMH = ArchiveGroup | Archive | Dir | File;

enum LRB {
  Local = 0,
  Remote = 1,
  Both = 2
}

function merge(target:(ArchiveGroup|Archive)[],from:(ArchiveGroup|Archive)[]) {
  from.forEach(f => {
    const old = target.find(o => o.id === f.id);
    if (old && old instanceof ArchiveGroup && f instanceof ArchiveGroup) {
      merge(old.children,f.children);
      old.lr = LRB.Both;
      old.update();
    } else if (!old) {
      target.push(f);
    }
  });
}

class ArchiveGroup extends vscode.TreeItem {
  id:string;
  lr:LRB;
  parent:ArchiveGroup|undefined;
  children:(ArchiveGroup | Archive)[];
  downloadable=false;
  constructor(group:FLAMS.ArchiveGroup,lr:LRB,parent?:ArchiveGroup) {
    const name = group.id.split("/").pop();
    if (!name) {
      throw new Error("𝖥𝖫∀𝖬∫: Invalid archive group name");
    }
    super(name,vscode.TreeItemCollapsibleState.Collapsed);
    this.id = group.id;
    this.parent = parent;
    this.children = [];
    this.lr = lr;
    this.iconPath = (lr === LRB.Local || lr === LRB.Both) ? 
      new vscode.ThemeIcon("library") : 
      vscode.Uri.joinPath(getContext().vsc.extensionUri,"img","MathHub.svg")
    ;
    if (lr === LRB.Remote) {this.downloadable = true;}
    this.contextValue = (lr === LRB.Remote || lr === LRB.Both) ? "remote" : "local";
  }

  update() {
    if (this.lr === LRB.Both && this.children.map(c => c.downloadable).includes(true)) {
      this.downloadable = true;
      this.contextValue = "remote";
    }
  }
}
class Archive extends vscode.TreeItem {
  id:string;
  local:boolean;
  parent:ArchiveGroup|undefined;
  downloadable:boolean;
  constructor(archive:FLAMS.Archive,local:boolean,downloadable:boolean,parent?:ArchiveGroup,mhs?:string[]) {
    const name = archive.id.split("/").pop();
    if (!name) {
      throw new Error("𝖥𝖫∀𝖬∫: Invalid archive name");
    }
    super(name,vscode.TreeItemCollapsibleState.Collapsed);
    this.id = archive.id;
    this.local = local;
    this.downloadable = downloadable;
    this.parent = parent;
    this.iconPath = local ? 
      new vscode.ThemeIcon("book") : 
      vscode.Uri.joinPath(getContext().vsc.extensionUri,"img","MathHub.svg")
    ;
    this.contextValue = downloadable ? "remote" : "local";
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

  constructor(archive:Archive,dir:FLAMS.Directory) {
    const name = dir.rel_path.split("/").pop();
    if (!name) {
      throw new Error("𝖥𝖫∀𝖬∫: Invalid directory name");
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
  constructor(archive:Archive,file:FLAMS.File) {
    const name = path.basename(file.rel_path);
    super(name,vscode.TreeItemCollapsibleState.None);
    this.archive = archive;
    this.id = `[${archive.id}]${file.rel_path}`;
    this.rel_path = file.rel_path;
    this.iconPath = new vscode.ThemeIcon("file");
    if (archive.resourceUri) {
      this.contextValue = "file";
      this.resourceUri = vscode.Uri.file(
        this.rel_path.split("/").reduce(
          (p,seg) => path.join(p,seg),
          path.join(archive.resourceUri.fsPath,"source")
        )
      );
      this.command = {
          command:Commands.openFile,
          title:"Open File",
          arguments:[this.resourceUri]
      };
    }
  }
}