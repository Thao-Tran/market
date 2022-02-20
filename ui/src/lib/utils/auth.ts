export function isUserLoggedIn() {
	return document.cookie.includes('market.loggedIn');
}
