FROM pmcvay1003/aicontainer:sha-3d06a69
WORKDIR /
COPY docker-requirements.txt .
RUN pip install -r docker-requirements.txt
COPY . .
WORKDIR /futr
# RUN python /futr/Transcription.py
EXPOSE 7437
ENTRYPOINT ["python3", "DB.py"]



