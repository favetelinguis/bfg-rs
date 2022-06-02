Config should be located in `$HOME/bfg/{demo|live}/config.json` this is also where results are written.

#### Markets:
- IX.D.DAX.IFMM.IP (Tyskland 40 Cash 1Eur)
- IX.D.FTSE.IFE.IP (FTSE 100 Cash 1Eur)
- IX.D.CAC.IMF.IP (Frankrike 40 Cash 1Eur)
- IX.D.NASDAQ.IFE.IP (US 100 Tech Cash 1Eur)
- IX.D.SPTRD.IFE.IP (USA 500 Cash 1Eur)
- IX.D.OMX.IFM.IP (Sverige 30 Cash 20Sek)

#### Limitations
Due to the fact that the CONFS subscription always send last conf at start i filter away all messages older then 10 seconds.
To get the correct behavior wait 10seconds between restarts of TUI.

Since Date is set once at start the system can not run over nigh and assume trading the next day. It has to be restarted each night.

## Before live
- Change the multiple of minimum range its 20 need to be some reasonable with regards to risk i can take.
- Remove q as a quit command and requre only ctrl-c for quit