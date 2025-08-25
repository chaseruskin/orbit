import multiprocessing as mp
import time
import os

def process_function_1():
    os.system('orbit b --top adder -t gui')
    pass


def process_function_2():
    # os.system('orbit b --top adder -t gui')
    time.sleep(0.010)
    os.system('orbit install --url https://github.com/hyperspace-labs/amp/archive/refs/tags/1.3.0.zip --force')
    pass


def main():
    # Create the first process
    p1 = mp.Process(target=process_function_1)

    # Create the second process
    p2 = mp.Process(target=process_function_2)

    print('STARTING PROCESSES')
    # Start both processes
    p1.start()
    p2.start()

    # Wait for both processes to complete
    p1.join()
    p2.join()

    print("PROCESSES COMPLETE")

if __name__ == "__main__":
    main()