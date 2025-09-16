# -*- coding: utf-8 -*
#!/usr/bin/python
#These Tools Take A Lot Of Time So Please Be Patient
#My Friendo : JametKNTLS -  h0d3_g4n - Moslem And All Coders
#Created Date = January - 18 - 2k22
# Blog : https://www.blog-gan.org          
#DONATE ME :(
	# BTC = 31mtLHqhaXXyCMnT2EU73U8fwYwigiEEU1
	# PERFECT MONEY  = U22270614
import requests, re, urllib2, os, sys, codecs, random,time
from multiprocessing.dummy import Pool					     	
from time import time as timer
from colorama import Fore								

def Banner():
    clear = '\x1b[0m'
    colors = [36, 32, 34, 35, 31, 37]

    x = '''
    _____  
^..^     \9
(oo)_____/ 
   WW  WW
1 . EXTRACT EMAIL
2 . DUMPERONES.CO VALIDATOR               
'''
    for N, line in enumerate(x.split('\n')):
        sys.stdout.write('\x1b[1;%dm%s%s\n' % (random.choice(colors), line, clear))
        time.sleep(0.02)
Banner()

choose = raw_input(':~# \033[34mChoose\033[32m Number : ')

def Extract():
	try:
		oni = raw_input(' \033[34mExtact\033[32m Email : ')
		with open(oni, 'r') as saan:
			for neesan in saan:
				okasan = neesan.strip()
				moushindeiru = re.findall("[A-Za-z0-9_%+-.]+@[A-Za-z0-9.-]+\.[A-Za-z]{2,5}", okasan)
				for omaewa in moushindeiru:
					print(Fore.GREEN+'RESULT EMAIL : ' +omaewa+Fore.WHITE)
					open('ResMail.txt', 'a').write(omaewa+'\n')
	except:
		pass

def Gmass(gms):
	try:
		head = {'User-Agent': 'Mozilla/5.0 (Linux; Android 11; Redmi Note 9 Pro) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/90.0.4430.210 Mobile Safari/537.36',
		'Accept': 'text/html,application/xhtml+xml,application/xml;q=0.9,image/avif,image/webp,image/apng,*/*;q=0.8,application/signed-exchange;v=b3;q=0.9',
		'Cookie': 'GMassUniqueID=558d3296-b37b-4cb8-8c7a-3e188c22e793; GMassAffiliateID='
		}
		#You Can Replace Key With Your Key Here &key=here By Registering At Gmass.co
		saan = requests.get('https://verify.gmass.co/verify?email='+gms+'&key=3c5c8060-844a-4cb5-ae6e-b6615971ef4d', headers=head).text
		if '"SMTPCode":250' in saan:
			print(Fore.YELLOW+'[ OK --> ]' + Fore.GREEN + gms +Fore.WHITE )
			open('Mail_OK.txt', 'a').write(gms+'\n')
		if '"SMTPCode":550' in saan:
			print(Fore.YELLOW+'[ FAILED --> ]' + Fore.RED + gms +Fore.WHITE )
			open('Mail_FAILED.txt', 'a').write(gms+'\n')
	except:
		pass

def Main():
	try:
		if choose =='1':
			Extract()
		elif choose == '2':
			list = raw_input("\n\033[91mDomain List\033[97m:~# \033[97m")
			gms = open(list, 'r').read().splitlines()
			pp = Pool(50)
			pr = pp.map(Gmass, gms)
	except:
		pass

if __name__ == '__main__':
	Main()
