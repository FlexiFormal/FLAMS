"use strict";
var __defProp = Object.defineProperty;
var __getOwnPropDesc = Object.getOwnPropertyDescriptor;
var __getOwnPropNames = Object.getOwnPropertyNames;
var __hasOwnProp = Object.prototype.hasOwnProperty;
var __export = (target, all) => {
  for (var name in all)
    __defProp(target, name, { get: all[name], enumerable: true });
};
var __copyProps = (to, from, except, desc) => {
  if (from && typeof from === "object" || typeof from === "function") {
    for (let key of __getOwnPropNames(from))
      if (!__hasOwnProp.call(to, key) && key !== except)
        __defProp(to, key, { get: () => from[key], enumerable: !(desc = __getOwnPropDesc(from, key)) || desc.enumerable });
  }
  return to;
};
var __toCommonJS = (mod) => __copyProps(__defProp({}, "__esModule", { value: true }), mod);

// src/index.ts
var index_exports = {};
__export(index_exports, {
  FLAMSServer: () => FLAMSServer
});
module.exports = __toCommonJS(index_exports);
var FLAMSServer = class {
  constructor(url) {
    this._url = url;
  }
  get url() {
    return this._url;
  }
  /**
   * All institutions and `archive.json`-registered documents
   */
  async index() {
    return await this.rawPostRequest("api/index", {});
  }
  /**
   * Full-text search for documents, assuming the given filter
   */
  async searchDocs(query, filter, numResults) {
    return await this.rawPostRequest("api/search", {
      query,
      opts: filter,
      num_results: numResults
    });
  }
  /**
   * Full-text search for (definitions of) symbols
   */
  async searchSymbols(query, numResults) {
    return await this.rawPostRequest("api/search_symbols", {
      query,
      num_results: numResults
    });
  }
  /**
   * List all archives/groups in the given group (or at top-level, if undefined)
   */
  async backendGroupEntries(in_entry) {
    return await this.rawPostRequest("api/backend/group_entries", {
      in: in_entry
    });
  }
  /**
   * List all directories/files in the given archive at path (or at top-level, if undefined)
   */
  async backendArchiveEntries(archive, in_path) {
    return await this.rawPostRequest("api/backend/archive_entries", {
      archive,
      path: in_path
    });
  }
  /**
   * SPARQL query
   */
  async query(sparql) {
    return await this.rawPostRequest("api/backend/query", { query: sparql });
  }
  /**
   * Get all dependencies of the given archive (excluding meta-inf archives)
   */
  async archiveDependencies(archives) {
    return await this.rawPostRequest("api/backend/archive_dependencies", {
      archives
    });
  }
  /**
   * Return the TOC of the given document
   */
  async contentToc(uri) {
    return await this.rawGetRequest("content/toc", uri);
  }
  /**
   * Get all learning objects for the given symbol; if problems === true, this includes Problems and Subproblems;
   * otherwise, only definitions and examples.
   */
  async learningObjects(uri, problems) {
    const exc = problems ? problems : false;
    const sym = "uri" in uri ? { uri: uri.uri, problems: exc } : { a: uri.a, p: uri.p, m: uri.m, s: uri.s, problems: exc };
    return await this.rawGetRequest("content/los", sym);
  }
  /**
   * Get the quiz in the given document.
   */
  async quiz(uri) {
    return await this.rawGetRequest("content/quiz", uri);
  }
  /**
   * Return slides for the given document / section
   */
  async slides(uri) {
    return await this.rawGetRequest("content/slides", uri);
  }
  /**
   * Batch grade an arrray of <solution,response[]> pairs.
   * Each of the responses will be graded against the corresponding solution, and the resulting
   * feedback returned at the same position. If *any* of the responses is malformed,
   * the whole batch will fail.
   * A SolutionData[] can be obtained from Solutions.to_solutions(). A ProblemFeedbackJson
   * can be turned into a "proper" ProblemFeedback using ProblemFeedback.from_json().
   */
  async batchGrade(...submissions) {
    return await this.rawPostRequest("content/grade", { submissions });
  }
  /**
   * Get the solution for the problem with the given URI. As string, so it can be
   * deserialized by the ts binding for the WASM datastructure
   */
  async solution(uri) {
    let r = await this.getRequestI("content/solution", uri);
    if (r) {
      return await r.text();
    }
  }
  async omdoc(uri) {
    return await this.rawGetRequest("content/omdoc", { uri });
  }
  async contentDocument(uri) {
    return await this.rawGetRequest("content/document", uri);
  }
  async contentFragment(uri) {
    return await this.rawGetRequest("content/fragment", uri);
  }
  async rawGetRequest(endpoint, request) {
    const response = await this.getRequestI(endpoint, request);
    if (response) {
      const j = await response.json();
      console.log("Response", endpoint, ":", j);
      return j;
    }
  }
  async getRequestI(endpoint, request) {
    const encodeParam = (v) => {
      return encodeURIComponent(JSON.stringify(v));
    };
    const buildQueryString = (obj, prefix = "") => {
      const params = [];
      if (obj === null || obj === void 0) {
        return params;
      }
      if (Array.isArray(obj)) {
        if (prefix) {
          params.push(`${prefix}=${encodeParam(obj)}`);
        }
      } else if (typeof obj === "string") {
        params.push(`${prefix}=${encodeURIComponent(obj)}`);
      } else if (typeof obj === "object" && !(obj instanceof Date)) {
        if (prefix) {
          params.push(`${prefix}=${encodeParam(obj)}`);
        } else {
          for (const [key, value] of Object.entries(obj)) {
            const newPrefix = prefix ? `${prefix}[${key}]` : key;
            params.push(...buildQueryString(value, newPrefix));
          }
        }
      } else {
        const value = obj instanceof Date ? obj.toISOString() : obj;
        params.push(`${prefix}=${encodeParam(value)}`);
      }
      return params;
    };
    const queryString = buildQueryString(request).join("&");
    const url = `${this._url}/${endpoint}${queryString ? "?" + queryString : ""}`;
    console.log("Calling", url);
    const response = await fetch(url, {
      method: "GET",
      headers: {
        Accept: "application/json"
      }
    });
    if (response.ok) {
      return response;
    }
  }
  async rawPostRequest(endpoint, request) {
    const response = await this.postRequestI(endpoint, request);
    if (response) {
      const j = await response.json();
      console.log(`Response ${this._url}/${endpoint} with body:`, j);
      return j;
    }
  }
  async postRequestI(endpoint, request) {
    const formData = new URLSearchParams();
    const appendToForm = (obj, prefix = "") => {
      if (Array.isArray(obj)) {
        obj.forEach((v, i) => appendToForm(v, `${prefix}[${i}]`));
      } else if (obj instanceof Date) {
        formData.append(prefix, obj.toISOString());
      } else if (obj && typeof obj === "object" && !(obj instanceof File)) {
        for (const [key, value] of Object.entries(obj)) {
          const newPrefix = prefix ? `${prefix}[${key}]` : key;
          appendToForm(value, newPrefix);
        }
      } else if (obj !== void 0 && obj !== null) {
        formData.append(prefix, String(obj));
      }
    };
    appendToForm(request);
    console.log(`Calling ${this._url}/${endpoint} with body`, formData);
    const response = await fetch(`${this._url}/${endpoint}`, {
      method: "POST",
      headers: {
        "Content-Type": "application/x-www-form-urlencoded"
      },
      body: formData.toString()
    });
    if (response.ok) {
      return response;
    }
  }
};
// Annotate the CommonJS export names for ESM import in node:
0 && (module.exports = {
  FLAMSServer
});
//# sourceMappingURL=index.js.map