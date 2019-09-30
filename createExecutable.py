from cx_Freeze import setup, Executable

setup(name = "pngtosvg" ,
      version = "0.1" ,
      description = "" ,
      executables = [Executable("pngtosvg.py")])