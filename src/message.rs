pub fn save_screen() {
	// TODO
	// FILE *fp;
	// short i, j, row, col;
	// char buf[DCOLS+2];
	// boolean found_non_blank;
	//
	//
	// if ((fp = fopen("player.rogue.screen", "w")) != NULL) {
	// 	for (i = 0; i < DROWS; i++) {
	// 		found_non_blank = 0;
	// 		for (j = (DCOLS - 1); j >= 0; j--) {
	// 			buf[j] = mvinch(i, j);
	// 			if (!found_non_blank) {
	// 				if ((buf[j] != ' ') || (j == 0)) {
	// 					buf[j + ((j == 0) ? 0 : 1)] = 0;
	// 					found_non_blank = 1;
	// 				}
	// 			}
	// 		}
	// 		fputs(buf, fp);
	// 		putc('\n', fp);
	// 	}
	// 	fclose(fp);
	// } else {
	// 	sound_bell();
	// }
}

pub fn sound_bell() {
	// TODO
	// putchar(7);
	// fflush(stdout);
}
