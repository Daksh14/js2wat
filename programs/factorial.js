function fact(n) {
	let result = 1;
	while (n > 0) {
		result = result * n;
		n = n - 1;
	}
	
	return result;
}


fact(5); // must return 120
