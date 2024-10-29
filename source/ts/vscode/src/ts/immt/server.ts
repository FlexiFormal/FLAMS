import * as immt from './types';

export class IMMTServer {
  _url:string;
  constructor(url:string) {
    this._url = url;
  }

  get url(): string {
    return this._url;
  }

  /// requires login
  async api_settings(): Promise<immt.Settings | undefined> {
    const ret = <[immt.Settings,any] | undefined> await this.postRequest("api/settings",{});
    if (ret) {
      const [settings,_] = ret;
      return settings;
    }
  }


  /// sets a cookie, i.e. only makes sense in a client-side context
  async api_login(username:string,password:string): Promise<void> {
    await this.postRequest("api/login",{username:username,password:password});
  }

  async api_login_state(): Promise<immt.LoginState | undefined> {
    return await this.postRequest("api/login_state",{});
  }

  async backend_group_entries(in_entry?:string): Promise<[immt.ArchiveGroup[],immt.Archive[]] | undefined> {
    return await this.postRequest("api/backend/group_entries",{in:in_entry});
  }

  async backend_archive_entries(archive:string,in_path?:string): Promise<[immt.Directory[],immt.File[]] | undefined> {
    return await this.postRequest("api/backend/archive_entries",{archive:archive,path:in_path});
  }

  async query(sparql:String): Promise<any> {
    return await this.postRequest("api/backend/query",{query:sparql});
  }

  async content_document(uri:immt.DocumentURIParams):Promise<[immt.DocumentURI,immt.CSS[],string] | undefined> {
    const arg = (uri instanceof immt.DocumentURI)? { uri: uri.uri } : uri;
    const ret = <[string,immt.CSS[],string] | undefined> await this.getRequest("content/document",arg);
    if (ret) {
      const [s,c,h] = ret;
      return [new immt.DocumentURI(s),c,h];
    }
  }

  async content_fragment(uri:immt.URIParams):Promise<[immt.CSS[],string] | undefined> {
    const arg = (uri instanceof immt.DocumentURI)? { uri: uri.uri } : uri;
    return await this.getRequest("content/fragment",arg);
  }

  async content_toc(uri:immt.DocumentURIParams):Promise<[immt.CSS[],immt.TOCElem[]] | undefined> {
    const arg = (uri instanceof immt.DocumentURI)? { uri: uri.uri } : uri;
    return await this.getRequest("content/toc",arg);
  }

  private async getRequest<TRequest extends Record<string,unknown>, TResponse>(endpoint:string,request:TRequest): Promise<TResponse | undefined> {
    const encodeParam = (v:unknown):string => {
      return encodeURIComponent(JSON.stringify(v));
    };
    const buildQueryString = (obj:unknown,prefix = ''): string[] => {
      const params: string[] = [];
      if (obj === null || obj === undefined) { return params; }
      if (Array.isArray(obj)) {
        if (prefix) {
          params.push(`${prefix}=${encodeParam(obj)}`);
        }
      } else if (typeof obj === 'string') {
        params.push(`${prefix}=${encodeURIComponent(obj)}`);
      } else if (typeof obj === 'object' && !(obj instanceof Date)) {
        if (prefix) {
          params.push(`${prefix}=${encodeParam(obj)}`);
        } else {
          for (const [key,value] of Object.entries(obj)) {
            const newPrefix = prefix ? `${prefix}[${key}]` : key;
            params.push(...buildQueryString(value,newPrefix));
          }
        }
      } else {
        const value = obj instanceof Date ? obj.toISOString() : obj;
        params.push(`${prefix}=${encodeParam(value)}`);
      }
      return params;
    };

    const queryString = buildQueryString(request).join('&');
    const url = `${this._url}/${endpoint}${queryString ? '?' + queryString : ''}`;
    console.log(url);
    const response = await fetch(url, {
      method: 'GET',
      headers: {
        'Accept': 'application/json'
      }
    });
    if (response.ok) {
      return await response.json() as TResponse;
    }
  }

  private async postRequest<TRequest extends Record<string,unknown>, TResponse>(endpoint:string,request:TRequest): Promise<TResponse | undefined> {
    const formData = new URLSearchParams();
    const appendToForm = (obj:unknown, prefix=''): void => {
      if (Array.isArray(obj)) {
        obj.forEach((v,i) => appendToForm(v,`${prefix}[${i}]`));
      } else if (obj instanceof Date) {
        formData.append(prefix, obj.toISOString());
      } else if (obj && typeof obj === 'object' && !(obj instanceof File)) {
        for (const [key,value] of Object.entries(obj)) {
          const newPrefix = prefix ? `${prefix}[${key}]` : key;
          appendToForm(value,newPrefix);
        }
      } else if (obj !== undefined && obj !== null) {
        formData.append(prefix, String(obj));
      }
    };
    appendToForm(request);
    const response = await fetch(`${this._url}/${endpoint}`, {
      method: 'POST',
      headers: {
        'Content-Type': 'application/x-www-form-urlencoded'
      },
      body: formData.toString()
    });

    if (response.ok) {
      return await response.json() as TResponse;
    }
  }
}

type IsEqual<T1,T2> = (T1 | T2) extends (T1 & T2) ? true : never;