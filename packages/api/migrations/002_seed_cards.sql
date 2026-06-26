-- 14-card starter deck: 7 bonuses, 7 curses
INSERT INTO cards (name, card_type, effect, flavor_text) VALUES
    ('Safe House',     'bonus',  'Move to any stop within 2 km without it counting as a new hiding location for 15 minutes.',           'The best hideout is the one no one suspects.'),
    ('Ghost Mode',     'bonus',  'Seekers may not ask a distance question for the next 10 minutes.',                                   'You are the wind.'),
    ('Decoy Signal',   'bonus',  'Give a false answer to one distance question. The following question must be answered truthfully.',   'Misdirection is an art form.'),
    ('Extra Time',     'bonus',  'Add 10 minutes to the current hiding round.',                                                        'Time is the ultimate luxury.'),
    ('Fog of War',     'bonus',  'Remove one exclusion zone of your choice from the seeker map.',                                      'Uncertainty is your ally.'),
    ('Detour',         'bonus',  'Seekers must travel to a specific stop named by the hider before continuing their search.',          'Send them on a wild goose chase.'),
    ('Radio Silence',  'bonus',  'Seekers cannot ask any questions for 8 minutes.',                                                    'Static is the sweetest sound.'),
    ('Slow Down',      'curse',  'You must wait 5 minutes before moving from your current location.',                                  'Your legs feel like lead.'),
    ('Compass Rose',   'curse',  'You must immediately reveal which cardinal quadrant (N/S/E/W) of the map you are in.',               'Point the way... to your doom.'),
    ('Crowded Stop',   'curse',  'Your next hiding location must be at a stop that has at least 2 named routes passing through.',      'The busier, the better — for them.'),
    ('Paparazzi',      'curse',  'Post a photo to the group chat hinting at your surroundings. No location metadata allowed.',         'Smile for the camera!'),
    ('Speed Limit',    'curse',  'You may only travel by foot for the next 10 minutes.',                                               'No shortcuts today.'),
    ('Wrong Turn',     'curse',  'You must move at least 1 km in the opposite direction from your current trajectory.',               'Left? No, right. Definitely right.'),
    ('Open Book',      'curse',  'Seekers receive one free distance question. You must answer honestly and cannot play a card in response.', 'Transparency has its costs.');
