/// <reference types="@sveltejs/kit" />

// Define heather-js types because the package hasn't published types yet
declare module 'heather-js' {
  interface ApiClientRequest {
    url: string;
    method: string;
    headers: Record<string, string>;
    data: any;
    meta: any;
  }

  export default class JsonApiClient {
    constructor(endpoint: any);

    define(model: any);
    buildRequestCreate: (ctx: {
      resource: any;
      type?: string;
      attributes?: any;
    }) => ApiClientRequest;
    buildRequestDelete: (ctx: { resource?: any; type?: string }) => ApiClientRequest;
    buildRequestFind: (ctx: { type: any }) => ApiClientRequest;
  }
}
