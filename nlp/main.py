import json
from kiwipiepy import Kiwi
from kiwipiepy.utils import Stopwords
kiwi = Kiwi()
stopwords = Stopwords()

def main():
    while True:
        try:
            sentence = input()
            result = []
            for token in kiwi.tokenize(sentence, stopwords=stopwords):
                if token.tag.startswith('NNG') or token.tag == 'SL':
                    result.append(token.form)
            print(json.dumps({
                "data": result,
                }))
        except EOFError as e:
            exit(0)

if __name__ == "__main__":
    main()

