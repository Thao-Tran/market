import { Token } from '$lib/models';
import JsonApiClient from 'heather-js';

export function getApiClient(): JsonApiClient {
	const client = new JsonApiClient(import.meta.env.VITE_BACKEND);
	client.define(Token);
	return client;
}
