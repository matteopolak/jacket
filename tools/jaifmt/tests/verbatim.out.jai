TEXT :: #string END
   keep   this   
	and this	
END;
CR :: #string,cr DONE
line
DONE

f :: () {
    s := "  string   contents  ";
    c := #char " ";
    #asm {
    mov.q   rax, rbx;
           add.q rax, 1;
    }
    // jaifmt: off
      keep   =   this;
         and_this   (   );
      // jaifmt: on
    back = 1;
}
