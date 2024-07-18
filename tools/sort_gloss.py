# Author: Chase Ruskin
# Details:
#   Sorts the glossary page into alphabetical order.


GLOSS_PATH = './docs/src/glossary.md'

WORD_SYMBOL = '### '


def main():
    map = {}

    with open(GLOSS_PATH, 'r') as gloss:
        # separate by '###'
        content = gloss.read()
        entries = content.split(WORD_SYMBOL)
        for e in entries:
            if len(e.strip()) == 0:
                continue
            # the word is the first
            word, define = e.split('\n', 1)
            if word.startswith('#'):
                continue
            map[word.strip()] = define
        pass

    sorted = list(map.keys())
    sorted.sort()

    for word in sorted:
        define = map[word]
        print(WORD_SYMBOL+word)
        print(define, end='')
    pass


if __name__ == '__main__':
    main()