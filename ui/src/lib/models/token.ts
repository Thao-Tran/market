export default class Token {
	id = ''; // Not used but necessary for jsonapi-rust
	email?: string;
	password?: string;

	constructor(email?: string, password?: string) {
		this.email = email;
		this.password = password;
	}
}
