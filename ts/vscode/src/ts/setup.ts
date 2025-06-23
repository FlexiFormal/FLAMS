import path from "path";
import { FLAMSPreContext, launch_local } from "../extension";
import { Settings } from "./commands";
import { add_exe, download, unzip } from "./utils";
import { REQUIRED_FLAMS, REQUIRED_STEX } from "./versions";
import * as vscode from 'vscode';
import * as fs from 'fs';

export async function setup(context:FLAMSPreContext): Promise<void> {
  await check_flams(context);
}

async function check_flams(context:FLAMSPreContext) {
  let versions = context.versions;
  if (!versions?.flams_path) {
    await flams_missing(context);
  } else {
    let v = await versions.flamsVersion();
    if (v) {
      if (v.newer_than(REQUIRED_FLAMS)) {
        await check_stex(context);
      } else {
        await flamsVersionMismatch(context);
      }
    } else { await flamsInvalid(context); }
  }
}

async function check_stex(context:FLAMSPreContext) {
  let versions = context.versions;
  if (await versions?.hasLatex()) {
    if (await versions?.hasSTeX()) {
      let v = await versions?.stexVersion();
      if (v) {
        if (v.newer_than(REQUIRED_STEX)) {
          launch_local(context);
        } else {
          vscode.window.showErrorMessage(`𝖥𝖫∀𝖬∫: Outdated stex package version`, { 
            modal: true,
            detail: `The 𝖥𝖫∀𝖬∫ extension requires at least version ${REQUIRED_STEX.toString()}, \
but your version is ${v.toString()}.
Please update your stex package version.`
          });
        }
      } else {
        vscode.window.showErrorMessage(`𝖥𝖫∀𝖬∫: Error determining stex package version`, { modal: true });
      }
    } else {
      vscode.window.showErrorMessage(`𝖥𝖫∀𝖬∫: No sTeX found!`, { modal: true,detail:"Make sure the stex package is installed" });
    }
  } else {
    vscode.window.showErrorMessage(`𝖥𝖫∀𝖬∫: No LaTeX found!`, { modal: true, detail:"Make sure pdflatex and kpsewhich are in your path" });
  }
}

async function flamsVersionMismatch(context:FLAMSPreContext) {
  await flamsProblem(
    '𝖥𝖫∀𝖬∫: Version outdated',
    `This version requires at least version ${REQUIRED_FLAMS.toString()}. \
You can either set the path to an up-to-date executable in the settings, \
or download it automatically from https://github.com/KWARC/FLAMS`,
    context
  );
}

async function flams_missing(context:FLAMSPreContext) {
  await flamsProblem(
    '𝖥𝖫∀𝖬∫: Path to executable not set',
    `An 𝖥𝖫∀𝖬∫ executable is required to run 𝖥𝖫∀𝖬∫. \
You can either set the path to the executable in the settings, \
or download it automatically from https://github.com/KWARC/FLAMS`,
  context
  );
}

async function flamsInvalid(context:FLAMSPreContext) {
  await flamsProblem('𝖥𝖫∀𝖬∫: executable invalid',
    `Your path to the 𝖥𝖫∀𝖬∫ executable does not point to a 𝖥𝖫∀𝖬∫ executable. \
You can either set a different path in the settings, \
or download it automatically from https://github.com/KWARC/FLAMS`,
    context
  );
}

async function flamsProblem(msg:string,long:string,context:FLAMSPreContext) {
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
      title: "Select 𝖥𝖫∀𝖬∫ executable",
      filters: {
        'Executables': process.platform.startsWith("win")?["exe"]:[]
      }
    }).then((uri) =>{ if (uri) { updateFlams(uri[0].fsPath,context); } });
  } else if (selection === DOWNLOAD) {
    downloadFlams(context);
  }
}

function updateFlams(path:string,context:FLAMSPreContext) {
  vscode.workspace.getConfiguration("flams").update(Settings.FlamsPath, path, vscode.ConfigurationTarget.Global)
  .then(() => {
    let versions = context.versions;
    versions?.reset();
    check_flams(context);
  });
}

async function downloadFlams(context:FLAMSPreContext) {
  const dir = await vscode.window.showOpenDialog({
    canSelectFiles: false,
    canSelectFolders: true,
    canSelectMany: false,
    title: "Select 𝖥𝖫∀𝖬∫ directory"
  });
  if (dir) {
    await downloadFromGithub(dir[0].fsPath,context);
  }
}

async function downloadFromGithub(dir:string,context:FLAMSPreContext) {
  vscode.window.withProgress({
    location: vscode.ProgressLocation.Notification,
    title: "Installing 𝖥𝖫∀𝖬∫",
    cancellable: false
  }, async (progress, _token) => {
    progress.report({ message: "Querying github.com" });
    const { Octokit } = await import('@octokit/rest');
    const octokit = new Octokit();
    const releases = await octokit.repos.listReleases({ owner: 'KWARC', repo: 'FLAMS', per_page: 3 });
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
      const zipfile = path.join(dir,"flams.zip");
      progress.report({ message: `Downloading ${url}` });
      const dl = await download(url,zipfile);
      if (!dl) { return; }
      progress.report({ message: `Unzipping ${zipfile}` });
      const zip = await unzip(zipfile,dir,[],["settings.toml"],[add_exe("flams")],progress);
      if (!zip) { return; }
      progress.report({ message: `Removing ${zipfile}` });
      fs.unlink(zipfile,err => {});
      updateFlams(add_exe(path.join(dir,"flams")),context);
    } else {
      vscode.window.showErrorMessage(`𝖥𝖫∀𝖬∫: Error downloading from github.com`);
    }
  });
}