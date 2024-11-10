# Kompilator

Kompilator składa się z dwóch komponentów: transpilatora z języka instant do kodu pośredniego i właściwego kompilatora z kodu pośredniego do LLVM lub JVM.

## Transpilator (Haskell)

Transpiluje kod w języku instant w kod, który jest prostszy do sparsowania, konkretnie do postaci tekstowej drzewa AST wygenerowanego przez BNFC.

Użyte biblioteki/narzędzia:
- BNFC

## Kompilator (Rust)

Kompiluje kod pośredni do LLVM lub JVM. Cel kompilacji wybiera się przez podanie odpowiedniej opcji podczas kompilowania kodu przez cargo. Implementuje oczekiwanie optymalizacje do JVM.

Użyte biblioteki/narzędzia:
- peg - biblioteka do parsowania
- ariadne - biblioteka do wypisywania błędów ze wskazaniem miejsca w kodzie, gdzie błąd wystąpił (głównie dodane z myślą o latte)

Struktura:

- error.rs - obsługa raportowania błedów
- common.rs - typy/struktury wspólne dla różnych plików
- parser.rs - parser kodu pośredniego
- compiler/* - kompilator do jvm/llvm

## Uruchomienie

Zbudowanie kompilatora wymaga cargo. Można go zainstalować na studentsie przez `make install-cargo`.

Inne reguły:
- `make` - zbuduj kompilator
- `make clean` - wyczyść pliki

#### Uwaga

Jeśli gramatyka wygenerowana przez transpilator posiada głęboko zagnieżdzone wyrażenia (rzędu > 10^4), być może dojdzie do przepełnienia stosu. Nieduże zwiększenie limitu rozmiaru stosu powinno pomóc.