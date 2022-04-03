import hashlib
import glob, os, sys

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
            sum = hashlib.sha256(body_bytes).hexdigest()
            print(sum, os.path.basename(pkg))


if __name__ == "__main__":
    main()