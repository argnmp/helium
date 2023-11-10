import json
from kiwipiepy import Kiwi
kiwi = Kiwi()

def main():
    while True:
        try:
            sentence = input()
            result = []
            for token in kiwi.tokenize(sentence):
                if token.tag.startswith('NN') or token.tag == 'SL':
                    result.append(token.form)
            print(json.dumps({
                "data": result,
                }))
        except EOFError as e:
            exit(0)

if __name__ == "__main__":
    main()
