# ------------------------------------------------------------------------------
# File: sum.py
# Author: Chase Ruskin
# Abstract:
#   Compute the checksum for a list of files found from glob matching a pattern.
#   The output will resemble the following:
#   '''
#   8852e7f180e9bc0821b0136a859617a9588bd636fdac9612c066550203f1e8c9 lib.rs
#   67cf113292aedfdb788e63da973c5de0d2ae4dc1c649cb3718dddbb9f6a5dd7f main.rs
#   '''
# Usage:    
#   python sum.py <pattern>
# Args:
#   <pattern>   a filepath pattern to collect a common set files
# ------------------------------------------------------------------------------
import hashlib
import unittest
import glob, os, sys
from typing import List

def compute_sha256(data: bytes) -> str:
    '''Compute the sha256 using built-in library function.'''
    return hashlib.sha256(data).hexdigest()


def main():
    if len(sys.argv) != 2:
        exit("error: enter a pattern to compute sha256")

    pattern = sys.argv[1]
    pkgs = glob.glob(pattern)

    if len(pkgs) == 0:
        exit("error: found zero matches for",pattern)

    for pkg in pkgs:
        with open(pkg, 'rb') as f:
            body_bytes = f.read()
            sum = compute_sha256(body_bytes)
            print(sum, os.path.basename(pkg))
    pass


if __name__ == "__main__":
    main()


class Test(unittest.TestCase):
    def test_sha256(self):
        # note: these test cases align with ones found in the rust implementation to ensure compatibility and correctness
        self.assertEqual(compute_sha256(b'hello world'), \
            'b94d27b9934d3e08a52e52d7da7dabfac484efe37a5380ee9088f7ace2efcde9')

        text = """\
Tyger Tyger, burning bright, 
In the forests of the night; 
What immortal hand or eye, 
Could frame thy fearful symmetry? 

In what distant deeps or skies. 
Burnt the fire of thine eyes? 
On what wings dare he aspire? 
What the hand, dare seize the fire? 

And what shoulder, & what art, 
Could twist the sinews of thy heart? 
And when thy heart began to beat, 
What dread hand? & what dread feet? 

What the hammer? what the chain, 
In what furnace was thy brain? 
What the anvil? what dread grasp, 
Dare its deadly terrors clasp! 

When the stars threw down their spears 
And water'd heaven with their tears: 
Did he smile his work to see? 
Did he who made the Lamb make thee? 

Tyger Tyger burning bright, 
In the forests of the night: 
What immortal hand or eye, 
Dare frame thy fearful symmetry?"""
        self.assertEqual(compute_sha256(bytes(text.encode())), \
            '0d732bb7f24e68fb3858646ba33bc9ce3240def191cde285a3f03ad1f763f52d')
        pass
    pass