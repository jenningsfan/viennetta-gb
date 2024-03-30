SEG_LEN = len("A:01 F:Z-HC BC:0013 DE:00d8 HL:014d SP:fffe PC:0100")

with open("log_comparer/correct.txt") as correct:
    for i in range(7):
        next(correct)
		
    with open("log_comparer/viennetta.txt") as vienneta:
        for (i, (correct_line, vienneta_line)) in enumerate(zip(correct, vienneta)):
            correct_line = correct_line[0:SEG_LEN].upper()
            vienneta_line = vienneta_line.strip().upper()

            if correct_line != vienneta_line:
                print(f"Line {i + 1}")
                print(f"Correct: {correct_line}")
                print(f"Viennetta: {vienneta_line}")
                break