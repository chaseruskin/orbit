import os

def main():
    print(os.environ.keys())
    url = os.environ['ORBIT_IP_SOURCE']
    print(url)
    pass


if __name__ == "__main__":
    main()