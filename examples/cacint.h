/* ----------------------------------------------------------------------
 *  System      : ChemApp
 * ----------------------------------------------------------------------
 *  Module      : cacint.h (ChemApp C/C++ interface)
 *
 * ----------------------------------------------------------------------
 *  Revision    : $Revision: 2571 $	
 *  Last Change : $Date: 2014-05-14 12:15:50 +0200 (Wed, 14 May 2014) $ by $Author: sp $  
 *
 *  Language    : C
 * ----------------------------------------------------------------------
 *  Subject     : This file contains the C/C++ interface to ChemApp 
 *                (header file)
 * ----------------------------------------------------------------------
 */

/*
This file is Copyright (C) GTT-Technologies, Herzogenrath, Germany.
It may only be used together with GTT-Technologies´ ChemApp software.
Unauthorised duplication and distribution both in printed and online
form, also in parts, is prohibited.
*/

/* ChemApp DLL function prototypes */

#ifndef _cacint
#define _cacint

#define DB  double		/* double	*/
#define DBP double*		/* double*	*/
#define CHP char*		/* char*	*/
#define VDP void*	


#ifndef LIP
#if (defined(_WIN64)||defined(WIN64)||defined(_X86_64)||defined(__x86_64__)||defined(x86_64))
#define LIP int*
#else
#define LIP long*
#endif // _WIN64 
#endif

#ifndef LI
#if (defined(_WIN64)||defined(WIN64)||defined(_X86_64)||defined(__x86_64__)||defined(x86_64))
#define LI int
#else
#define LI long
#endif // _WIN64 
#endif

#ifndef LNT
#if (defined(_WIN64)||defined(WIN64)||defined(_X86_64)||defined(__x86_64__)||defined(x86_64))
#define LNT size_t		
#else
#define LNT long
#endif // _WIN64 
#endif

#ifndef CMT
#if (defined(UNIX)||defined(Unix)||defined(__CYGWIN__))
#define CMT extern int
#else
#define CMT void __stdcall
#endif // _WIN64 
#endif

#ifdef UNIX
#if(defined(_X86_64)||defined(__x86_64__)||defined(x86_64))
#define ftnlen int
#else
#define ftnlen long
#endif
#endif

#ifdef __CYGWIN__
#if(defined(_X86_64)||defined(__x86_64__)||defined(x86_64))
#define ftnlen int
#else
#define ftnlen long
#endif
#endif

//#ifdef UNIX
//#define LI  int          	
//#define LIP int*       	
//#define LNT int        	
//#define CMT extern int		
//#define VDP void*		
//#define ftnlen int			/* FORTRAN string length type */
//
//#else
//#define LI  long         		/* unsigned int		*/
//#define LIP long*       		/* unsigned int*	*/
//#define LNT long        		/* unsigned int		*/
//#define CMT void __stdcall	/* void __stdcall	*/
//#define VDP void*				/* void*			*/
//#endif
  

/* Length of a TQ String */
#define TQSTRLEN 25


/* Macro for defining TQStrings */
#define TQSTRING(x) char x[TQSTRLEN]
/* TQ Error Message Buffer */
extern char TQERRMSG[3][80]; 

#ifdef __cplusplus
extern "C" {
#endif
int tqini (LIP NOERR);
int tqopen(CHP FILE, LI LUN, LIP NOERR);
int tqclos(LI LUN, LIP NOERR);
int tqgio (CHP OPTION, LIP IVAL, LIP NOERR);
int tqcio (CHP OPTION, LI IVAL, LIP NOERR);
int tqrfil(LIP NOERR);
int tqgsu (CHP OPTION, CHP UNIT, LIP NOERR);
int tqcsu (CHP OPTION, CHP UNIT, LIP NOERR);
int tqinsc(CHP NAME, LIP INDEXS, LIP NOERR);
int tqgnsc(LI INDEXS, CHP NAME, LIP NOERR);
int tqnosc(LIP NSCOM, LIP NOERR);
int tqstsc(LI INDEXS, DBP STOI, DBP WMASS, LIP NOERR);
int tqcsc (CHP NAME, LIP NOERR);
int tqinp (CHP NAME, LIP INDEXP, LIP NOERR);
int tqgnp (LI INDEXP, CHP NAME, LIP NOERR);
int tqnop (LIP NPHASE, LIP NOERR);
int tqinpc(CHP NAME, LI INDEXP, LIP INDEXC, LIP NOERR);
int tqgnpc(LI INDEXP, LI INDEXC, CHP NAME, LIP NOERR);
int tqnopc(LI INDEXP, LIP NPCON, LIP NOERR);
int tqstpc(LI INDEXP, LI INDEXC, DBP STOI, DBP WMASS, LIP NOERR);
int tqgsp (LI INDEXP, CHP OPTION, LIP NOERR);
int tqcsp (LI INDEXP, CHP OPTION, LIP NOERR);
int tqgspc(LI INDEXP, LI INDEXC, CHP OPTION, LIP NOERR);
int tqcspc(LI INDEXP, LI INDEXC, CHP OPTION, LIP NOERR);
int tqsetc(CHP OPTION, LI INDEXP, LI INDEX, DB VAL, LIP NUMCON, LIP NOERR);
int tqremc(LI NUMCON, LIP NOERR);
int tqsttp(CHP IDENTS, DBP VALS, LIP NOERR);
int tqstca(CHP IDENTS, LI INDEXP, LI INDEXC, DB VAL, LIP NOERR);
int tqstec(CHP OPTION, LI INDEXP, DB VAL, LIP NOERR);
int tqstrm(CHP IDENTS, LIP NOERR);
int tqce  (CHP OPTION, LI INDEXP, LI INDEXC, DBP VALS, LIP NOERR);
int tqcel (CHP OPTION, LI INDEXP, LI INDEXC, DBP VALS, LIP NOERR);
int tqclim(CHP OPTION, DB VAL, LIP NOERR);
int tqgetr(CHP OPTION, LI INDEXP, LI INDEX, DBP VAL, LIP NOERR);
int tqgdpc(CHP OPTION, LI INDEXP, LI INDEXC,DBP VAL, LIP NOERR);
int tqshow(LIP NOERR);
int tqerr (CHP MESS, LIP NOERR);
int tqcprt(LIP NOERR);
int tqvers(LIP NVERS, LIP NOERR);
int tqsize(LIP NA,LIP NB,LIP NC,LIP ND,LIP NE,LIP NF,LIP NG,LIP NH,LIP NI,LIP NJ,LIP NK,LIP NOERR);
int tqmodl(LI INDEXP, CHP NAME, LIP NOERR);
int tqstxp(CHP IDENTS,CHP OPTION, DBP VAL, LIP NOERR);
int tqlite(LIP LITE, LIP NOERR);
int tqrbin(LIP NOERR);
int tqmap(CHP OPTION, LI INDEXP, LI INDEXC, DBP VALS, LIP ICONT, LIP NOERR);
int tqmapl(CHP OPTION, LI INDEXP, LI INDEXC, DBP VALS, LIP ICONT, LIP NOERR);
int tqpcis(LI INDEXP, LI INDEXC, LIP ISPERM, LIP NOERR);
int tqopna(CHP FILE, LI LUN, LIP NOERR);
int tqopnb(CHP FILE, LI LUN, LIP NOERR);
int tqnosl(LI INDEXP, LIP NSUBL, LIP NOERR);
int tqnolc(LI INDEXP, LI INDEXL, LIP NSLCON, LIP NOERR);
int tqinlc(CHP NAME, LI INDEXP, LI INDEXL, LIP INDEXC, LIP NOERR);
int tqgnlc(LI INDEXP, LI INDEXL, LI INDEXC, CHP NAME, LIP NOERR);
int tqgtlc(LI INDEXP, LI INDEXL, LI INDEXC, DBP VAL, LIP NOERR);
/*
int tqgopn (CHP FILE,LI LUN,CHP FFORM,CHP FSTAT,CHP FACC,LI RECL,
	    LIP IOSTAT,LIP NOERR);
*/
int tqbond(LI INDEXP, LI INDEXA, LI INDEXB, LI INDEXC, LI INDEXD, 
	   DBP VAL, LIP NOERR);
int tqused(LIP NA,LIP NB,LIP NC,LIP ND,LIP NE,LIP NF,LIP NG,LIP NH,LIP NI,
	   LIP NJ,LIP NK,LIP NOERR);
int tqgtrh(LIP TFHVER,
	   CHP TFHNWP,
	   LIP TFHVNW,
	   CHP TFHNRP,
	   LIP TFHVNR,
	   LIP TFHDTC,
	   LIP TFHDTE,
	   CHP TFHID,
	   CHP TFHUSR,
	   CHP TFHREM,
	   LIP NOERR);
int tqopnt(CHP FILE, LI LUN, LIP NOERR);
int tqrcst(LIP NOERR);
int tqgtid(CHP ID, LIP NOERR);
int tqgtnm(CHP NAME, LIP NOERR);
int tqgtpi(CHP PID, LIP NOERR);
int tqwstr(CHP OPTION, CHP CTXT, LIP NOERR);
int tqgted(LIP EDMON, LIP EDYEAR, LIP NOERR);
int tqgthi(CHP HASPT, LIP HASPID, LIP NOERR);
int tqcen (CHP OPTION, LI INDEXP, LI INDEXC, DBP VALS, LIP NOERR);
int tqcenl(CHP OPTION, LI INDEXP, LI INDEXC, DBP VALS, LIP NOERR);
int tqwasc(CHP FILE, LIP NOERR);
int tqcdat(LI I1, LI I2, LI I3, LI I4, LI I5, DB VAL, LIP NOERR);
int tqchar(LI INDEXP, LI INDEXC, DBP VAL, LIP NOERR);
int tqcnsc(LI INDEXS, CHP NAME, LIP NOERR);
int tqlpar(LI INDEXP, CHP OPTION, LIP NOPAR, CHP CHRPAR, LIP LGTPAR, LIP NOERR);
int tqgpar(LI INDEXP, CHP OPTION, LI INDEXX, LIP NOEXPR, LIP NVALA, DBP VALA, LIP NOERR);
int tqgdat(LI INDEXP, LI INDEXC, CHP OPTION, LI INDEXR, LIP NVALV, DBP VALV, LIP NOERR);
int tqconf(CHP OPTION, LI INDEXA, LI INDEXB, LI INDEXC, LIP NOERR);

#ifdef __cplusplus
};
#endif		/* __cplusplus	*/
#endif		/* _cacint		*/
