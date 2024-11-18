import path from "path";
import { IMMTPreContext, launch_local } from "../extension";
import { Settings } from "./commands";
import { add_exe, download, unzip } from "./utils";
import { REQUIRED_IMMT, REQUIRED_STEX } from "./versions";
import * as vscode from 'vscode';
import * as fs from 'fs';

export async function setup(context:IMMTPreContext): Promise<void> {
  await check_immt(context);
}

async function check_immt(context:IMMTPreContext) {
  let versions = context.versions;
  if (!versions?.immt_path) {
    await immt_missing(context);
  } else {
    let v = await versions.immtversion();
    if (v) {
      if (v.newer_than(REQUIRED_IMMT)) {
        await check_stex(context);
      } else {
        await immt_version_mismatch(context);
      }
    } else { await immt_invalid(context); }
  }
}

async function check_stex(context:IMMTPreContext) {
  let versions = context.versions;
  if (await versions?.hasLatex()) {
    if (await versions?.hasSTeX()) {
      let v = await versions?.stexversion();
      if (v) {
        if (v.newer_than(REQUIRED_STEX)) {
          launch_local(context);
        } else {
          vscode.window.showErrorMessage(`iMMT: Outdated stex package version`, { 
            modal: true,
            detail: `The iMMT extension requires at least version ${REQUIRED_STEX.toString()}, \
but your version is ${v.toString()}.
Please update your stex package version.`
          });
        }
      } else {
        vscode.window.showErrorMessage(`iMMT: Error determining stex package version`, { modal: true });
      }
    } else {
      vscode.window.showErrorMessage(`iMMT: No sTeX found!`, { modal: true,detail:"Make sure the stex package is installed" });
    }
  } else {
    vscode.window.showErrorMessage(`iMMT: No LaTeX found!`, { modal: true, detail:"Make sure pdflatex and kpsewhich are in your path" });
  }
}

async function immt_version_mismatch(context:IMMTPreContext) {
  await immt_problem(
    'iMMT: Version outdated',
    `This version requires at least version ${REQUIRED_IMMT.toString()}. \
You can either set the path to an up-to-date executable in the settings, \
or download it automatically from https://github.com/KWARC/iMMT`,
    context
  );
}

async function immt_missing(context:IMMTPreContext) {
  await immt_problem(
    'iMMT: Path to executable not set',
    `An iMMT executable is required to run iMMT. \
You can either set the path to the executable in the settings, \
or download it automatically from https://github.com/KWARC/iMMT`,
  context
  );
}

async function immt_invalid(context:IMMTPreContext) {
  await immt_problem('iMMT: executable invalid',
    `Your path to the iMMT executable does not point to an iMMT executable. \
You can either set a different path in the settings, \
or download it automatically from https://github.com/KWARC/iMMT`,
    context
  );
}

async function immt_problem(msg:string,long:string,context:IMMTPreContext) {
  const SET_PATH = "Set path";
  const DOWNLOAD = "Download";
  const selection = await vscode.window.showInformationMessage(msg, { modal: true,
    detail:long 
  },DOWNLOAD,SET_PATH);
  if (selection === SET_PATH) {
    vscode.window.showOpenDialog({
      canSelectFiles: true,
      canSelectFolders: false,
      canSelectMany: false,
      title: "Select iMMT executable",
      filters: {
        'Executables': process.platform.startsWith("win")?["exe"]:[]
      }
    }).then((uri) =>{ if (uri) { update_immt(uri[0].fsPath,context); } });
  } else if (selection === DOWNLOAD) {
    download_immt(context);
  }
}

function update_immt(path:string,context:IMMTPreContext) {
  vscode.workspace.getConfiguration("immt").update(Settings.ImmtPath, path, vscode.ConfigurationTarget.Global)
  .then(() => {
    let versions = context.versions;
    versions?.reset();
    check_immt(context);
  });
}

async function download_immt(context:IMMTPreContext) {
  const dir = await vscode.window.showOpenDialog({
    canSelectFiles: false,
    canSelectFolders: true,
    canSelectMany: false,
    title: "Select iMMT directory"
  });
  if (dir) {
    await download_from_github(dir[0].fsPath,context);
  }
}

async function download_from_github(dir:string,context:IMMTPreContext) {
  vscode.window.withProgress({
    location: vscode.ProgressLocation.Notification,
    title: "Installing iMMT",
    cancellable: false
  }, async (progress, _token) => {
    progress.report({ message: "Querying github.com" });
    const { Octokit } = await import('@octokit/rest');
    const octokit = new Octokit();
    const releases = await octokit.repos.listReleases({ owner: 'KWARC', repo: 'iMMT', per_page: 3 });
    const release = releases.data.values().next().value;

    let filename = "linux.zip";
    switch (process.platform) {
      case "win32": 
        filename = "windows.zip";
        break;
      case "darwin": 
        filename = "mac.zip";
      default: break;
    }

    const url = release?.assets.find((a) => a.name === filename)?.browser_download_url;
    if (url) {
      const zipfile = path.join(dir,"immt.zip");
      progress.report({ message: `Downloading ${url}` });
      const dl = await download(url,zipfile);
      if (!dl) { return; }
      progress.report({ message: `Unzipping ${zipfile}` });
      const zip = await unzip(zipfile,dir,[],["settings.toml"],["immt"],progress);
      if (!zip) { return; }
      progress.report({ message: `Removing ${zipfile}` });
      fs.unlink(zipfile,err => {});
      update_immt(add_exe(path.join(dir,"immt")),context);
    } else {
      vscode.window.showErrorMessage(`iMMT: Error downloading from github.com`);
    }
  });
}