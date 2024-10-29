import * as vscode from 'vscode';
import { call_cmd } from './utils';
import * as fs from "fs";
import { Settings } from './commands';

export class Versions {
  private _latex:boolean | undefined;
	private _stex_path: string | undefined;
	private _stex_version : Version | undefined;
	private _immt_version : Version | undefined;

  constructor() {}

  get immt_path(): string | undefined {
		const config = vscode.workspace.getConfiguration("immt");
		return config.get<string>(Settings.ImmtPath)?.trim();
	}

	get stex_path(): string | undefined {
		return this._stex_path;
	}

  async hasLatex(): Promise<boolean> {
		if (this._latex) {return true;}
		let res = await call_cmd("kpsewhich",["--version"]);
		this._latex = (res !== undefined);
		return this._latex;
	}
  async hasSTeX(): Promise<boolean> {
		if (this._stex_path) {return true;}
		let res = await call_cmd("kpsewhich",["stex.sty"]);
		if (res) {
			this._stex_path = res.trim();
			this._latex = true;
			return true;
		}
		return false;
	}

  async isValid(): Promise<boolean> {
		let immt = await this.immtversion();
		let stex = await this.stexversion();
		return immt !== undefined && stex !== undefined && 
      immt.newer_than(REQUIRED_IMMT) && 
      stex.newer_than(REQUIRED_STEX);
	}

	async stexversion(): Promise<Version | undefined> {
		if (this._stex_version) {return this._stex_version;}
		await this.hasSTeX();
		if (this._stex_path) {
			let ret = fs.readFileSync(this._stex_path).toString();
			const regex = /\\message{\^\^J\*~This~is~sTeX~version~(\d+\.\d+\.\d+)~\*\^\^J}/;
			const match = ret.match(regex);
			if (match) {
				const vstring = match[1];
				this._stex_version = new Version(vstring);
				return this._stex_version;
			}
		}
	}

  async immtversion(): Promise<Version | undefined> {
		if (this._immt_version) {return this._immt_version; }
    let path = this.immt_path;
    if (path) {
      let res = await call_cmd(path,["--version"]);
      if (res) {
			  const regex = /immt (\d+\.\d+\.\d+)/;
        const match = res.match(regex);
        if (match) {
          const vstring = match[1];
          this._immt_version = new Version(vstring);
          return this._immt_version;
        }
      }
    }
	}

  reset() {
		this._immt_version = undefined;
	}
}

export class Version {
	major : number;
	minor : number;
	revision: number;
	constructor(s:string | [number,number,number]) {
		if (typeof(s) === "string") {
			const match = s.trim().split('.');
			if (match.length ===1 ) {
				[this.major,this.minor,this.revision] = [parseInt(match[0]),0,0];
			} else if (match.length === 2) {
				[this.major,this.minor,this.revision] = [parseInt(match[0]),parseInt(match[1]),0];
			} else {
				[this.major,this.minor,this.revision] = [parseInt(match[0]),parseInt(match[1]),parseInt(match[2])];
			}
		} else {
			[this.major,this.minor,this.revision] = s;
		}
	}
	toString() : string {
		return this.major.toString() + "." + this.minor.toString() + "." + this.revision.toString();
	}
	newer_than(that: Version):boolean {
		return this.major > that.major || (
			this.major === that.major && (
				this.minor > that.minor || (
					this.minor === that.minor && this.revision >= that.revision
				)
			)
		);
	}
}

export const REQUIRED_IMMT = new Version([0,0,1]);
export const REQUIRED_STEX = new Version([4,0,0]);

