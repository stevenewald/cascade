#include <fstream>
#include <iterator>
#include <vector>
#include <iostream>

using namespace std;

int main() {
	ifstream input("index.table", ios::binary);
	vector<char> bytes(
        	(istreambuf_iterator<char>(input)),
        	(istreambuf_iterator<char>()));

	input.close();
	
	for(int i = 0; i < bytes.size(); i ++) {
		cout << (int) bytes[i] << " ";
	}

	return 1;
}
